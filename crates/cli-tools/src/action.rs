// Copyright 2024 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::borrow::Cow;
use std::fmt::Display;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::time::Duration;

use anyhow::{Context, Result, anyhow, bail, ensure};
use clap::{ValueEnum, ValueHint};
use rusb::GlobalContext;
use tokio::fs::File;
use tokio::io::{AsyncBufRead, AsyncBufReadExt as _, AsyncWrite, AsyncWriteExt, BufReader};
use tokio::process::Command;
use wasefire_common::platform::Side;
use wasefire_protocol::bundle::Bundle;
use wasefire_protocol::common::{AppletKind, Hexa, Name};
use wasefire_protocol::{self as service, ConnectionExt as _, DynDevice, applet};
use wasefire_wire::{self as wire, Yoke};

use crate::cargo::metadata;
use crate::error::root_cause_is;
use crate::{cmd, fs};

mod protocol;
pub mod usb_serial;

/// Options to connect to a platform.
#[derive(Clone, clap::Args)]
pub struct ConnectionOptions {
    /// How to connect to the platform.
    ///
    /// Possible values are:
    /// - usb (there must be exactly one connected platform on USB)
    /// - usb:SERIAL (the serial must be in hexadecimal)
    /// - usb:BUS:DEV
    /// - unix[:PATH] (defaults to /tmp/wasefire)
    /// - tcp[:HOST:PORT] (defaults to 127.0.0.1:3457)
    #[arg(long, default_value = "usb", env = "WASEFIRE_PROTOCOL", verbatim_doc_comment)]
    protocol: protocol::Protocol,

    /// Timeout to send or receive with the USB protocol.
    #[arg(long, default_value = "0s")]
    timeout: humantime::Duration,
}

impl ConnectionOptions {
    /// Establishes a connection.
    pub async fn connect(&self) -> Result<DynDevice> {
        self.protocol.connect(*self.timeout).await
    }

    /// Returns whether these options identify a device even after reboot.
    pub fn reboot_stable(&self) -> bool {
        match &self.protocol {
            protocol::Protocol::Usb(x) => match x {
                protocol::ProtocolUsb::Auto => true,
                protocol::ProtocolUsb::Serial(_) => true,
                protocol::ProtocolUsb::BusDev { .. } => false,
            },
            protocol::Protocol::Unix(_) => true,
            protocol::Protocol::Tcp(_) => true,
        }
    }
}

/// Installs an applet on a platform.
#[derive(clap::Args)]
pub struct AppletInstall {
    /// Path to the applet to install.
    #[arg(value_hint = ValueHint::FilePath)]
    pub applet: PathBuf,

    #[command(flatten)]
    pub transfer: Transfer,

    #[command(subcommand)]
    pub wait: Option<AppletInstallWait>,
}

#[derive(clap::Subcommand)]
pub enum AppletInstallWait {
    /// Waits until the applet exits.
    #[group(id = "AppletInstallWait::Wait")]
    Wait {
        #[command(flatten)]
        action: AppletExitStatus,
    },
}

impl AppletInstall {
    pub async fn run(self, device: &DynDevice) -> Result<()> {
        let AppletInstall { applet, transfer, wait } = self;
        let applet = fs::read(applet).await?;
        let applet = Bundle::decode(&applet)?.applet()?;
        let info = device.platform_info().await?;
        ensure!(info.applet_kind().is_none_or(|x| x == applet.kind()), "applet kind mismatch");
        let applet = applet.payload(device.version())?;
        transfer
            .run::<service::AppletInstall2>(device, applet, "Installed", None::<fn(_) -> _>)
            .await?;
        match wait {
            Some(AppletInstallWait::Wait { mut action }) => {
                action.wait.ensure_wait();
                action.run(device).await
            }
            None => Ok(()),
        }
    }
}

/// Uninstalls an applet from a platform.
#[derive(clap::Args)]
pub struct AppletUninstall {}

impl AppletUninstall {
    pub async fn run(self, device: &DynDevice) -> Result<()> {
        let AppletUninstall {} = self;
        let transfer = Transfer { dry_run: false };
        transfer
            .run::<service::AppletInstall2>(device, Vec::new(), "Erased", None::<fn(_) -> _>)
            .await
    }
}

/// Prints the metadata of an applet from a platform.
#[derive(clap::Args)]
pub struct AppletMetadata {}

impl AppletMetadata {
    pub async fn run(self, device: &DynDevice) -> Result<()> {
        let AppletMetadata {} = self;
        let metadata = device.call::<service::AppletMetadata0>(applet::AppletId).await?;
        println!("   name: {}", metadata.get().name);
        println!("version: {}", metadata.get().version);
        Ok(())
    }
}

/// Prints the exit status of an applet from a platform.
#[derive(clap::Parser)]
#[non_exhaustive]
pub struct AppletExitStatus {
    #[command(flatten)]
    pub wait: Wait,

    /// Also exits with the applet exit code.
    #[arg(long)]
    exit_code: bool,
}

impl AppletExitStatus {
    fn print(status: Option<applet::ExitStatus>) {
        match status {
            Some(status) => println!("{status}."),
            None => println!("The applet is still running."),
        }
    }

    fn code(status: Option<applet::ExitStatus>) -> i32 {
        match status {
            Some(applet::ExitStatus::Exit) => 0,
            Some(applet::ExitStatus::Abort) => 1,
            Some(applet::ExitStatus::Trap) => 2,
            Some(applet::ExitStatus::Kill) => 62,
            None => 63,
        }
    }

    pub fn ensure_exit(&mut self) {
        self.exit_code = true;
    }

    pub async fn run(self, device: &DynDevice) -> Result<()> {
        let AppletExitStatus { wait, exit_code } = self;
        let status = wait
            .run::<service::AppletExitStatus, applet::ExitStatus>(device, applet::AppletId)
            .await?
            .map(|x| *x.get());
        Self::print(status);
        if exit_code {
            std::process::exit(Self::code(status))
        }
        Ok(())
    }
}

/// Reboots an applet installed on a platform.
#[derive(clap::Args)]
pub struct AppletReboot {}

impl AppletReboot {
    pub async fn run(self, device: &DynDevice) -> Result<()> {
        let AppletReboot {} = self;
        device.call::<service::AppletReboot>(applet::AppletId).await.map(|x| *x.get())
    }
}

/// Parameters for an applet or platform RPC.
#[derive(clap::Args)]
struct Rpc {
    /// Reads the request from this file instead of standard input.
    #[arg(long, value_hint = ValueHint::FilePath)]
    input: Option<PathBuf>,

    /// Writes the response to this file instead of standard output.
    #[arg(long, value_hint = ValueHint::AnyPath)]
    output: Option<PathBuf>,

    /// Loops reading requests as lines and concatenating responses.
    #[arg(long)]
    repl: bool,
}

enum RpcState {
    One { input: Option<PathBuf>, output: Option<PathBuf>, read: bool },
    Loop { input: Pin<Box<dyn AsyncBufRead>>, output: Pin<Box<dyn AsyncWrite>> },
}

impl Rpc {
    async fn start(self) -> Result<RpcState> {
        let Rpc { input, output, repl } = self;
        if !repl {
            return Ok(RpcState::One { input, output, read: false });
        }
        let input: Pin<Box<dyn AsyncBufRead>> = match input {
            None => Box::pin(BufReader::new(tokio::io::stdin())),
            Some(path) => Box::pin(BufReader::new(File::open(path).await?)),
        };
        let output: Pin<Box<dyn AsyncWrite>> = match output {
            None => Box::pin(tokio::io::stdout()),
            Some(path) => Box::pin(File::create(path).await?),
        };
        Ok(RpcState::Loop { input, output })
    }
}

impl RpcState {
    async fn read(&mut self) -> Result<Option<Vec<u8>>> {
        match self {
            RpcState::One { read: x @ false, .. } => *x = true,
            RpcState::One { .. } => return Ok(None),
            RpcState::Loop { .. } => (),
        }
        Ok(Some(match self {
            RpcState::One { input: None, .. } => fs::read_stdin().await?,
            RpcState::One { input: Some(path), .. } => fs::read(path).await?,
            RpcState::Loop { input, .. } => {
                let mut line = String::new();
                if input.read_line(&mut line).await? == 0 {
                    return Ok(None);
                }
                line.into_bytes()
            }
        }))
    }

    async fn write(&mut self, response: &[u8]) -> Result<()> {
        match self {
            RpcState::One { output: None, .. } => fs::write_stdout(response).await,
            RpcState::One { output: Some(path), .. } => fs::write(path, response).await,
            RpcState::Loop { output, .. } => {
                output.write_all(response).await?;
                output.flush().await?;
                Ok(())
            }
        }
    }
}

/// Calls an RPC to an applet on a platform.
#[derive(clap::Args)]
pub struct AppletRpc {
    /// Applet identifier in the platform.
    applet: Option<String>,

    #[command(flatten)]
    rpc: Rpc,

    #[command(flatten)]
    wait: Wait,
}

impl AppletRpc {
    pub async fn run(self, device: &DynDevice) -> Result<()> {
        let AppletRpc { applet, rpc, mut wait } = self;
        let applet_id = match applet {
            Some(_) => bail!("applet identifiers are not supported yet"),
            None => applet::AppletId,
        };
        wait.ensure_wait();
        let mut rpc = rpc.start().await?;
        while let Some(request) = rpc.read().await? {
            let request = applet::Request { applet_id, request: Cow::Borrowed(&request) };
            device.call::<service::AppletRequest>(request).await?.get();
            match wait.run::<service::AppletResponse, Cow<[u8]>>(device, applet_id).await? {
                None => bail!("did not receive a response"),
                Some(response) => rpc.write(response.get()).await?,
            }
        }
        Ok(())
    }
}

/// Options to repeatedly call a command with an optional response.
#[derive(clap::Parser)]
pub struct Wait {
    /// Waits until there is a response.
    ///
    /// This is equivalent to --period=100ms.
    #[arg(long)]
    wait: bool,

    /// Retries every so often until there is a response.
    ///
    /// The command doesn't return `None` in that case.
    #[arg(long, conflicts_with = "wait")]
    period: Option<humantime::Duration>,
}

impl Wait {
    pub fn ensure_wait(&mut self) {
        if self.wait || self.period.is_some() {
            return;
        }
        self.wait = true;
    }

    pub fn set_period(&mut self, period: Duration) {
        self.wait = false;
        self.period = Some(period.into());
    }

    pub async fn run<S, T: wire::Wire<'static>>(
        &self, device: &DynDevice, request: S::Request<'_>,
    ) -> Result<Option<Yoke<T::Type<'static>>>>
    where S: for<'a> service::Service<Response<'a> = Option<T::Type<'a>>> {
        let Wait { wait, period } = self;
        let period = match (wait, period) {
            (true, None) => Some(Duration::from_millis(100)),
            (true, Some(_)) => unreachable!(),
            (false, None) => None,
            (false, Some(x)) => Some(**x),
        };
        let request = S::request(request);
        loop {
            match device.call_ref::<S>(&request).await?.try_map(|x| x.ok_or(())) {
                Ok(x) => break Ok(Some(x)),
                Err(()) => match period {
                    Some(period) => tokio::time::sleep(period).await,
                    None => break Ok(None),
                },
            }
        }
    }
}

/// Clears the store for the platform and all applets.
#[derive(clap::Args)]
pub struct PlatformClearStore {
    /// Clears all entries with a key greater or equal to this value.
    #[arg(default_value_t = 0)]
    min_key: usize,
}

impl PlatformClearStore {
    pub async fn run(self, device: &DynDevice) -> Result<()> {
        let PlatformClearStore { min_key } = self;
        device.call::<service::PlatformClearStore>(min_key).await.map(|x| *x.get())
    }
}

/// Prints information about a platform.
#[derive(clap::Args)]
pub struct PlatformInfo {}

impl PlatformInfo {
    pub async fn run(self, device: &DynDevice) -> Result<()> {
        let PlatformInfo {} = self;
        let info = device.platform_info().await?;
        println!("  serial: {}", info.serial());
        if let Some(applet_kind) = info.applet_kind() {
            println!("  applet: {}", applet_kind.name());
        }
        if let Some(running_side) = info.running_side() {
            println!("    side: {running_side}");
        }
        if let Some(running_name) = info.running_name() {
            println!("    name: {running_name}");
        }
        println!(" version: {}", info.running_version());
        if let Some(version) = info.opposite_version() {
            let version = match version {
                Ok(x) => x,
                Err(e) => return Ok(println!("opposite: {e}")),
            };
            println!("--- opposite side ---");
            if let Some(name) = info.opposite_name() {
                println!("    name: {}", name.unwrap());
            }
            println!(" version: {}", version);
        }
        Ok(())
    }
}

/// Lists the platforms connected on USB.
#[derive(clap::Args)]
pub struct PlatformList {
    /// Timeout to send or receive on the platform protocol.
    #[arg(long, default_value = "1s")]
    timeout: humantime::Duration,
}

impl PlatformList {
    pub async fn run(self) -> Result<()> {
        let PlatformList { timeout } = self;
        let context = GlobalContext::default();
        let candidates = wasefire_protocol_usb::list(&context)?;
        println!("There are {} connected platforms on USB:", candidates.len());
        for candidate in candidates {
            let connection = candidate.connect(*timeout)?;
            let serial = protocol::ProtocolUsb::Serial(protocol::serial(&connection).await?);
            let bus = connection.device().bus_number();
            let dev = connection.device().address();
            let busdev = protocol::ProtocolUsb::BusDev { bus, dev };
            println!("- {serial} or {busdev}");
        }
        Ok(())
    }
}

/// Updates a platform.
#[derive(clap::Args)]
pub struct PlatformUpdate {
    /// Path to the A side of the new platform.
    ///
    /// If only this file is provided, it is used without checking the running side. In particular,
    /// it can be the B side of the new platform.
    #[arg(value_hint = ValueHint::FilePath)]
    pub platform_a: PathBuf,

    /// Path to the B side of the new platform.
    ///
    /// If this file is not provided, [`Self::platform_a`] is used regardless of the running side.
    #[arg(value_hint = ValueHint::FilePath)]
    pub platform_b: Option<PathBuf>,

    #[command(flatten)]
    pub transfer: Transfer,
}

impl PlatformUpdate {
    pub async fn run(self, device: &DynDevice) -> Result<()> {
        let PlatformUpdate { platform_a, platform_b, transfer } = self;
        let platform = match platform_b {
            Some(platform_b) => match device.platform_info().await?.running_side() {
                None => bail!("device does not support platform update"),
                Some(Side::A) => platform_b,
                Some(Side::B) => platform_a,
            },
            None => platform_a,
        };
        let platform = fs::read(platform).await?;
        transfer
            .run::<service::PlatformUpdate>(
                device,
                platform,
                "Updated",
                Some(|_| bail!("device responded to a transfer finish")),
            )
            .await
    }
}

/// Parameters for a transfer from the host to the device.
#[derive(Clone, clap::Args)]
pub struct Transfer {
    /// Whether the transfer is a dry-run.
    #[arg(long)]
    dry_run: bool,
}

impl Transfer {
    async fn run<S>(
        self, device: &DynDevice, payload: Vec<u8>, message: &'static str,
        finish: Option<impl FnOnce(Yoke<S::Response<'static>>) -> Result<!>>,
    ) -> Result<()>
    where
        S: for<'a> service::Service<
                Request<'a> = service::transfer::Request<'a>,
                Response<'a> = service::transfer::Response,
            >,
    {
        use wasefire_protocol::transfer::{Request, Response};
        let Transfer { dry_run } = self;
        let Response::Start { chunk_size, num_pages } =
            *device.call::<S>(Request::Start { dry_run }).await?.get()
        else {
            bail!("received unexpected response");
        };
        let multi_progress = indicatif::MultiProgress::new();
        let style = indicatif::ProgressStyle::with_template(
            "{msg:9} {elapsed:>3} {spinner} [{wide_bar}] {bytes:>10} / {total_bytes:<10}",
        )?
        .tick_chars("-\\|/ ")
        .progress_chars("##-");
        let mut progress = None;
        if 0 < num_pages {
            let progress = progress.insert(
                multi_progress.add(indicatif::ProgressBar::new((num_pages * chunk_size) as u64)),
            );
            progress.set_style(style.clone());
            progress.set_message("Erasing");
            for _ in 0 .. num_pages {
                device.call::<S>(Request::Erase).await?.get();
                progress.inc(chunk_size as u64);
            }
        }
        if !payload.is_empty() {
            if let Some(progress) = progress.take() {
                progress.finish_with_message("Erased");
            }
            let progress = progress
                .insert(multi_progress.add(indicatif::ProgressBar::new(payload.len() as u64)));
            progress.set_style(style.clone());
            progress.set_message("Writing");
            for chunk in payload.chunks(chunk_size) {
                device.call::<S>(Request::Write { chunk: Cow::Borrowed(chunk) }).await?.get();
                progress.inc(chunk.len() as u64);
            }
        }
        let progress = progress.unwrap_or_else(|| indicatif::ProgressBar::new(0).with_style(style));
        progress.set_message("Finishing");
        match (dry_run, finish) {
            (false, Some(finish)) => final_call::<S>(device, Request::Finish, finish).await?,
            _ => drop(device.call::<S>(Request::Finish).await?.get()),
        }
        progress.finish_with_message(message);
        Ok(())
    }
}

async fn final_call<S: service::Service>(
    device: &DynDevice, request: S::Request<'_>,
    proof: impl FnOnce(Yoke<S::Response<'static>>) -> Result<!>,
) -> Result<()> {
    device.send(&S::request(request)).await?;
    match device.receive::<S>().await {
        Ok(x) => proof(x)?,
        Err(e) => {
            if root_cause_is::<rusb::Error>(&e, |x| {
                use rusb::Error::*;
                matches!(x, NoDevice | Pipe | Io)
            }) {
                return Ok(());
            }
            if root_cause_is::<std::io::Error>(&e, |x| {
                use std::io::ErrorKind::*;
                matches!(x.kind(), NotConnected | BrokenPipe | UnexpectedEof)
            }) {
                return Ok(());
            }
            Err(e)
        }
    }
}

/// Reboots a platform.
#[derive(clap::Args)]
pub struct PlatformReboot {}

impl PlatformReboot {
    pub async fn run(self, device: &DynDevice) -> Result<()> {
        let PlatformReboot {} = self;
        final_call::<service::PlatformReboot>(device, (), |x| match *x.get() {}).await
    }
}

/// Locks a platform.
#[derive(clap::Args)]
pub struct PlatformLock {}

impl PlatformLock {
    pub async fn run(self, device: &DynDevice) -> Result<()> {
        let PlatformLock {} = self;
        device.call::<service::PlatformLock>(()).await.map(|x| *x.get())
    }
}

/// Calls a vendor RPC on a platform.
#[derive(clap::Args)]
pub struct PlatformRpc {
    #[command(flatten)]
    rpc: Rpc,
}

impl PlatformRpc {
    pub async fn run(self, device: &DynDevice) -> Result<()> {
        let PlatformRpc { rpc } = self;
        let mut rpc = rpc.start().await?;
        while let Some(request) = rpc.read().await? {
            let request = Cow::Owned(request);
            let response = device.call::<service::PlatformVendor>(request).await?;
            rpc.write(response.get()).await?;
        }
        Ok(())
    }
}

/// Creates a new Rust applet project.
#[derive(clap::Args)]
pub struct RustAppletNew {
    /// Where to create the applet project.
    #[arg(value_hint = ValueHint::AnyPath)]
    path: PathBuf,

    /// Name of the applet project (defaults to the directory name).
    #[arg(long)]
    name: Option<String>,
}

impl RustAppletNew {
    pub async fn run(self) -> Result<()> {
        let RustAppletNew { path, name } = self;
        let mut cargo = Command::new("cargo");
        cargo.args(["new", "--lib"]).arg(&path);
        if let Some(name) = name {
            cargo.arg(format!("--name={name}"));
        }
        cmd::execute(&mut cargo).await?;
        cmd::execute(Command::new("cargo").args(["add", "wasefire"]).current_dir(&path)).await?;
        let mut cargo = Command::new("cargo");
        cargo.args(["add", "wasefire-stub", "--optional"]);
        cmd::execute(cargo.current_dir(&path)).await?;
        let mut sed = Command::new("sed");
        sed.arg("-i");
        sed.arg("s#^wasefire-stub\\( = .\"dep:wasefire-stub\"\\)#test\\1, \"wasefire/test\"#");
        sed.arg("Cargo.toml");
        cmd::execute(sed.current_dir(&path)).await?;
        tokio::fs::remove_file(path.join("src/lib.rs")).await?;
        fs::write(path.join("src/lib.rs"), include_str!("data/lib.rs")).await?;
        Ok(())
    }
}

/// Builds a Rust applet from its project.
#[derive(clap::Parser)]
pub struct RustAppletBuild {
    /// Builds for production, disabling debugging facilities.
    #[arg(long)]
    pub prod: bool,

    /// The name of the applet.
    #[arg(long, default_value_t)]
    pub name: Name<'static>,

    /// The version of the applet.
    #[arg(long, default_value_t)]
    pub version: Hexa<'static>,

    /// Builds a native applet, e.g. --native=thumbv7em-none-eabi.
    #[arg(long, value_name = "TARGET")]
    pub native: Option<String>,

    /// Builds a pulley applet.
    #[arg(long, conflicts_with = "native")]
    pub pulley: bool,

    /// Root directory of the crate.
    #[arg(long, value_name = "DIRECTORY", default_value = ".")]
    #[arg(value_hint = ValueHint::DirPath)]
    pub crate_dir: PathBuf,

    #[command(flatten)]
    pub output: BundleOutput,

    /// Cargo profile.
    #[arg(long, default_value = "release")]
    pub profile: String,

    /// Optimization level.
    #[arg(long, short = 'O')]
    pub opt_level: Option<OptLevel>,

    /// Stack size (ignored for native applets).
    #[arg(long, default_value = "16384")]
    pub stack_size: usize,

    /// Extra arguments to cargo, e.g. --features=foo.
    #[arg(last = true)]
    pub cargo: Vec<String>,
}

impl RustAppletBuild {
    pub async fn run(self) -> Result<PathBuf> {
        let metadata = metadata(&self.crate_dir).await?;
        let package = &metadata.packages[0];
        let target_dir =
            fs::try_relative(std::env::current_dir()?, &metadata.target_directory).await?;
        let name = package.name.replace('-', "_");
        let mut cargo = Command::new("cargo");
        nightly_toolchain(&mut cargo).await;
        let mut rustflags = Vec::new();
        cargo.args(["rustc", "--lib"]);
        // We deliberately don't use the provided profile for those configs because they don't
        // depend on user-provided options (as opposed to opt-level).
        cargo.arg("--config=profile.release.codegen-units=1");
        cargo.arg("--config=profile.release.lto=true");
        cargo.arg("--config=profile.release.panic=\"abort\"");
        match &self.native {
            None => {
                rustflags.push(format!("-C link-arg=-zstack-size={}", self.stack_size));
                rustflags.push("-C target-feature=+bulk-memory".to_string());
                cargo.args(["--crate-type=cdylib", "--target=wasm32-unknown-unknown"]);
                wasefire_feature(package, "wasm", &mut cargo)?;
            }
            Some(target) => {
                cargo.args(["--crate-type=staticlib", &format!("--target={target}")]);
                wasefire_feature(package, "native", &mut cargo)?;
                if target == "riscv32imc-unknown-none-elf" {
                    wasefire_feature(package, "unsafe-assume-single-core", &mut cargo)?;
                }
            }
        }
        let profile = &self.profile;
        cargo.arg(format!("--profile={profile}"));
        if let Some(level) = self.opt_level {
            cargo.arg(format!("--config=profile.{profile}.opt-level={level}"));
        }
        cargo.args(&self.cargo);
        if self.prod {
            cargo.arg("-Zbuild-std=core,alloc");
            // TODO(https://github.com/rust-lang/rust/issues/122105): Remove when fixed.
            rustflags.push("--allow=unused-crate-dependencies".to_string());
            if self.opt_level.is_some_and(OptLevel::optimize_for_size) {
                cargo.arg("-Zbuild-std-features=optimize_for_size");
            }
            cargo.arg(format!("--config=profile.{profile}.panic=\"immediate-abort\""));
        } else {
            cargo.env("WASEFIRE_DEBUG", "");
        }
        cargo.env("RUSTFLAGS", rustflags.join(" "));
        cargo.current_dir(&self.crate_dir);
        cmd::execute(&mut cargo).await?;
        if let Some(target) = &self.native {
            let src = target_dir.join(format!("{target}/{profile}/lib{name}.a"));
            let dst = self.output.path("libapplet.a").await?;
            if fs::has_changed(&src, &dst).await? {
                fs::copy(src, &dst).await?;
            }
            return Ok(dst);
        }
        let src = target_dir.join(format!("wasm32-unknown-unknown/{profile}/{name}.wasm"));
        let kind = if self.pulley { AppletKind::Pulley } else { AppletKind::Wasm };
        // We don't know if the name or version changed since last time, so we always create a new
        // bundle. This should be fast enough to not matter (and no further computation should
        // depend on the bundle file).
        self.output
            .bundle_applet(src, target_dir, self.opt_level, kind, self.name, self.version)
            .await
    }
}

/// Runs the unit-tests of a Rust applet project.
#[derive(clap::Args)]
pub struct RustAppletTest {
    /// Root directory of the crate.
    #[arg(long, value_name = "DIRECTORY", default_value = ".")]
    #[arg(value_hint = ValueHint::DirPath)]
    crate_dir: PathBuf,

    /// Extra arguments to cargo, e.g. --features=foo.
    #[arg(last = true)]
    cargo: Vec<String>,
}

impl RustAppletTest {
    pub async fn run(self) -> Result<()> {
        let metadata = metadata(&self.crate_dir).await?;
        let package = &metadata.packages[0];
        ensure!(package.features.contains_key("test"), "missing test feature");
        let mut cargo = Command::new("cargo");
        nightly_toolchain(&mut cargo).await;
        cargo.args(["test", "--features=test"]);
        cargo.args(&self.cargo);
        cargo.current_dir(&self.crate_dir);
        cmd::replace(cargo)
    }
}

/// Builds and installs a Rust applet from its project.
#[derive(clap::Parser)]
pub struct RustAppletInstall {
    #[command(flatten)]
    build: RustAppletBuild,

    #[command(flatten)]
    transfer: Transfer,

    #[command(subcommand)]
    wait: Option<AppletInstallWait>,
}

impl RustAppletInstall {
    pub async fn run(self, device: &DynDevice) -> Result<()> {
        let RustAppletInstall { build, transfer, wait } = self;
        let applet = build.run().await?;
        let install = AppletInstall { applet, transfer, wait };
        install.run(device).await
    }
}

/// Prints information about a bundle.
#[derive(clap::Args)]
pub struct BundleInfo {
    /// Path to the bundle to inspect.
    #[arg(value_hint = ValueHint::FilePath)]
    pub bundle: PathBuf,
}

impl BundleInfo {
    pub async fn run(self) -> Result<()> {
        let BundleInfo { bundle } = self;
        match Bundle::decode(&fs::read(bundle).await?)? {
            Bundle::Platform0(x) => {
                println!("Platform (version 0):");
                println!("- Name: {}", x.metadata.name);
                println!("- Version: {}", x.metadata.version);
                println!("- Side A: {} bytes", x.side_a.len());
                println!("- Side B: {} bytes", x.side_b.len());
            }
            Bundle::Applet0(x) => {
                println!("Applet (version 0):");
                println!("- Kind: {}", x.kind);
                println!("- Name: {}", x.metadata.name);
                println!("- Version: {}", x.metadata.version);
                println!("- Data: {} bytes", x.data.len());
            }
        }
        Ok(())
    }
}

#[derive(clap::Parser)]
pub struct BundleOutput {
    /// Where to write the bundle file.
    ///
    /// If the path does not exist and ends with a slash, it is assumed to be a directory. If the
    /// path is a directory, the file is written to that directory with some default name.
    #[arg(long, value_hint = ValueHint::AnyPath, default_value = "wasefire/")]
    pub output: PathBuf,

    /// Do not make parent directories as needed.
    #[arg(long)]
    pub no_parents: bool,
}

impl BundleOutput {
    /// Returns the path of the bundle file given its default name.
    pub async fn path(&self, name: impl AsRef<Path>) -> Result<PathBuf> {
        let mut path = self.output.clone();
        let is_dir = if fs::exists(&path).await {
            fs::metadata(&path).await?.is_dir()
        } else {
            path.has_trailing_sep()
        };
        if is_dir {
            path.push(name);
        }
        Ok(path)
    }

    /// Writes the bundle file given its default name and content.
    ///
    /// Returns the path of the written file.
    pub async fn write(&self, name: impl AsRef<Path>, content: &[u8]) -> Result<PathBuf> {
        let path = self.path(name).await?;
        if !self.no_parents {
            fs::create_parent(&path).await?;
        }
        fs::write(&path, content).await?;
        Ok(path)
    }

    /// Bundles and writes a WASM module as an applet.
    ///
    /// Returns the path of the written file.
    pub async fn bundle_applet(
        &self, wasm: impl AsRef<Path>, target: impl AsRef<Path>, opt_level: Option<OptLevel>,
        kind: AppletKind, name: Name<'_>, version: Hexa<'_>,
    ) -> Result<PathBuf> {
        let opt = target.as_ref().join("wasefire/applet-opt.wasm");
        optimize_wasm(wasm.as_ref(), opt_level, &opt).await?;
        let dst = target.as_ref().join(format!("wasefire/applet.{kind}"));
        match kind {
            AppletKind::Wasm => compute_sidetable(opt, &dst).await?,
            AppletKind::Pulley => compile_pulley(opt, &dst).await?,
            AppletKind::Native => bail!("cannot bundle a native applet"),
        }
        let metadata = applet::Metadata0 { name, version };
        let data = tokio::fs::read(dst).await?.into();
        let bundle = Bundle::Applet0(wasefire_protocol::bundle::Applet0 { kind, metadata, data });
        self.write("applet.wfb", &bundle.encode()?).await
    }
}

#[derive(Copy, Clone, ValueEnum)]
pub enum OptLevel {
    #[value(name = "0")]
    O0,
    #[value(name = "1")]
    O1,
    #[value(name = "2")]
    O2,
    #[value(name = "3")]
    O3,
    #[value(name = "s")]
    Os,
    #[value(name = "z")]
    Oz,
}

impl OptLevel {
    /// Returns whether the opt-level optimizes for size.
    pub fn optimize_for_size(self) -> bool {
        matches!(self, OptLevel::Os | OptLevel::Oz)
    }
}

impl Display for OptLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = self.to_possible_value().unwrap();
        let name = value.get_name();
        if f.alternate() || !matches!(self, OptLevel::Os | OptLevel::Oz) {
            write!(f, "{name}")
        } else {
            write!(f, "{name:?}")
        }
    }
}

async fn nightly_toolchain(cargo: &mut Command) {
    const TOOLCHAIN: &str = "nightly-2026-02-11";
    let mut rustup = Command::new("rustup");
    rustup.arg("--version");
    rustup.stdout(std::process::Stdio::null());
    rustup.stderr(std::process::Stdio::null());
    if rustup.status().await.is_ok_and(|x| x.success()) {
        cargo.arg(format!("+{TOOLCHAIN}"));
    }
}

/// Strips and optimizes a wasm module.
pub async fn optimize_wasm(
    src: impl AsRef<Path>, opt_level: Option<OptLevel>, dst: impl AsRef<Path>,
) -> Result<()> {
    if !fs::has_changed(src.as_ref(), dst.as_ref()).await? {
        return Ok(());
    }
    let result: Result<()> = try {
        fs::copy(src, dst.as_ref()).await?;
        cmd::execute(Command::new("wasm-strip").arg(dst.as_ref())).await?;
        let mut opt = Command::new("wasm-opt");
        opt.args(["--enable-bulk-memory", "--enable-sign-ext", "--enable-mutable-globals"]);
        match opt_level {
            Some(level) => drop(opt.arg(format!("-O{level:#}"))),
            None => drop(opt.arg("-O")),
        }
        opt.arg(dst.as_ref());
        opt.arg("-o");
        opt.arg(dst.as_ref());
        cmd::execute(&mut opt).await?;
    };
    if result.is_err() {
        let _ = fs::remove_file(dst).await;
    }
    result.context("optimizing wasm")
}

/// Compiles a WASM applet into a Pulley applet.
pub async fn compile_pulley(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> Result<()> {
    if !fs::has_changed(src.as_ref(), dst.as_ref()).await? {
        return Ok(());
    }
    let result: Result<()> = try {
        let wasm = fs::read(src).await?;
        let mut config = wasmtime::Config::new();
        config.target("pulley32")?;
        // TODO(https://github.com/bytecodealliance/wasmtime/issues/10286): Also strip symbol table.
        config.generate_address_map(false);
        config.memory_init_cow(false);
        config.memory_reservation(0);
        config.wasm_relaxed_simd(false);
        config.wasm_simd(false);
        let engine = wasmtime::Engine::new(&config)?;
        fs::write(dst.as_ref(), &engine.precompile_module(&wasm)?).await?;
    };
    if result.is_err() {
        let _ = fs::remove_file(dst).await;
    }
    result.context("compiling to pulley")
}

/// Computes the side-table and inserts it as the first section.
pub async fn compute_sidetable(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> Result<()> {
    if !fs::has_changed(src.as_ref(), dst.as_ref()).await? {
        return Ok(());
    }
    let result: Result<()> = try {
        let wasm = fs::read(src).await?;
        let wasm = wasefire_interpreter::prepare(&wasm)
            .map_err(|_| anyhow!("failed to compute side-table"))?;
        fs::write(&dst, &wasm).await?;
    };
    if result.is_err() {
        let _ = fs::remove_file(dst).await;
    }
    result
}

fn wasefire_feature(
    package: &cargo_metadata::Package, feature: &str, cargo: &mut Command,
) -> Result<()> {
    if package.features.contains_key(feature) {
        cargo.arg(format!("--features={feature}"));
    } else {
        ensure!(
            package.dependencies.iter().any(|x| x.name == "wasefire"),
            "wasefire must be a direct dependency"
        );
        cargo.arg(format!("--features=wasefire/{feature}"));
    }
    Ok(())
}
