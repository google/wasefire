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

use std::fmt::Display;
use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{Result, bail, ensure};
use clap::{ValueEnum, ValueHint};
use rusb::GlobalContext;
use tokio::process::Command;
use wasefire_common::platform::Side;
use wasefire_protocol::{self as service, Connection, ConnectionExt, applet};
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
    pub async fn connect(&self) -> Result<Box<dyn Connection>> {
        self.protocol.connect(*self.timeout).await
    }
}

/// Returns the API version of a platform.
#[derive(clap::Args)]
pub struct PlatformApiVersion {}

impl PlatformApiVersion {
    pub async fn run(self, connection: &mut dyn Connection) -> Result<u32> {
        let PlatformApiVersion {} = self;
        connection.call::<service::ApiVersion>(()).await.map(|x| *x.get())
    }
}

/// Installs an applet on a platform.
#[derive(clap::Args)]
pub struct AppletInstall {
    /// Path to the applet to install.
    #[arg(value_hint = ValueHint::FilePath)]
    pub applet: PathBuf,

    #[clap(flatten)]
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
    pub async fn run(self, connection: &mut dyn Connection) -> Result<()> {
        let AppletInstall { applet, transfer, wait } = self;
        transfer
            .run::<service::AppletInstall>(connection, applet, "Installed", None::<fn(_) -> _>)
            .await?;
        match wait {
            Some(AppletInstallWait::Wait { mut action }) => {
                action.wait.ensure_wait();
                action.run(connection).await
            }
            None => Ok(()),
        }
    }
}

/// Uninstalls an applet from a platform.
#[derive(clap::Args)]
pub struct AppletUninstall {}

impl AppletUninstall {
    pub async fn run(self, connection: &mut dyn Connection) -> Result<()> {
        let AppletUninstall {} = self;
        connection.call::<service::AppletUninstall>(()).await.map(|x| *x.get())
    }
}

/// Prints the exit status of an applet from a platform.
#[derive(clap::Parser)]
#[non_exhaustive]
pub struct AppletExitStatus {
    #[clap(flatten)]
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

    pub async fn run(self, connection: &mut dyn Connection) -> Result<()> {
        let AppletExitStatus { wait, exit_code } = self;
        let status = wait
            .run::<service::AppletExitStatus, applet::ExitStatus>(connection, applet::AppletId)
            .await?
            .map(|x| *x.get());
        Self::print(status);
        if exit_code {
            std::process::exit(Self::code(status))
        }
        Ok(())
    }
}

/// Parameters for an applet or platform RPC.
#[derive(clap::Args)]
pub struct Rpc {
    /// Reads the request from this file instead of standard input.
    #[arg(long, value_hint = ValueHint::FilePath)]
    input: Option<PathBuf>,

    /// Writes the response to this file instead of standard output.
    #[arg(long, value_hint = ValueHint::AnyPath)]
    output: Option<PathBuf>,
}

impl Rpc {
    async fn read(&self) -> Result<Vec<u8>> {
        match &self.input {
            Some(path) => fs::read(path).await,
            None => fs::read_stdin().await,
        }
    }

    async fn write(&self, response: &[u8]) -> Result<()> {
        match &self.output {
            Some(path) => fs::write(path, response).await,
            None => fs::write_stdout(response).await,
        }
    }
}

/// Calls an RPC to an applet on a platform.
#[derive(clap::Args)]
pub struct AppletRpc {
    /// Applet identifier in the platform.
    applet: Option<String>,

    #[clap(flatten)]
    rpc: Rpc,

    #[clap(flatten)]
    wait: Wait,
}

impl AppletRpc {
    pub async fn run(self, connection: &mut dyn Connection) -> Result<()> {
        let AppletRpc { applet, rpc, mut wait } = self;
        let applet_id = match applet {
            Some(_) => bail!("applet identifiers are not supported yet"),
            None => applet::AppletId,
        };
        let request = applet::Request { applet_id, request: &rpc.read().await? };
        connection.call::<service::AppletRequest>(request).await?.get();
        wait.ensure_wait();
        match wait.run::<service::AppletResponse, &[u8]>(connection, applet_id).await? {
            None => bail!("did not receive a response"),
            Some(response) => rpc.write(response.get()).await,
        }
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
        self, connection: &mut dyn Connection, request: S::Request<'_>,
    ) -> Result<Option<Yoke<T::Type<'static>>>>
    where S: for<'a> service::Service<Response<'a> = Option<T::Type<'a>>> {
        let Wait { wait, period } = self;
        let period = match (wait, period) {
            (true, None) => Some(Duration::from_millis(100)),
            (true, Some(_)) => unreachable!(),
            (false, None) => None,
            (false, Some(x)) => Some(*x),
        };
        let request = S::request(request);
        loop {
            match connection.call_ref::<S>(&request).await?.try_map(|x| x.ok_or(())) {
                Ok(x) => break Ok(Some(x)),
                Err(()) => match period {
                    Some(period) => tokio::time::sleep(period).await,
                    None => break Ok(None),
                },
            }
        }
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
            let mut connection = candidate.connect(*timeout)?;
            let serial = protocol::ProtocolUsb::Serial(protocol::serial(&mut connection).await?);
            let bus = connection.device().bus_number();
            let dev = connection.device().address();
            let busdev = protocol::ProtocolUsb::BusDev { bus, dev };
            println!("- {serial} or {busdev}");
        }
        Ok(())
    }
}

/// Returns information about a platform.
#[derive(clap::Args)]
pub struct PlatformInfo {}

impl PlatformInfo {
    pub async fn print(self, connection: &mut dyn Connection) -> Result<()> {
        Ok(print!("{}", self.run(connection).await?.get()))
    }

    pub async fn run(
        self, connection: &mut dyn Connection,
    ) -> Result<Yoke<service::platform::Info<'static>>> {
        let PlatformInfo {} = self;
        connection.call::<service::PlatformInfo>(()).await
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

    #[clap(flatten)]
    pub transfer: Transfer,
}

impl PlatformUpdate {
    pub async fn run(self, connection: &mut dyn Connection) -> Result<()> {
        let PlatformUpdate { platform_a, platform_b, transfer } = self;
        let platform = match platform_b {
            Some(platform_b) => match (PlatformInfo {}).run(connection).await?.get().running_side {
                Side::A => platform_b,
                Side::B => platform_a,
            },
            None => platform_a,
        };
        transfer
            .run::<service::PlatformUpdate>(
                connection,
                platform,
                "Updated",
                Some(|_| bail!("device responded to a transfer finish")),
            )
            .await
    }
}

/// Parameters for a transfer from the host to the device.
#[derive(clap::Args)]
pub struct Transfer {
    /// Whether the transfer is a dry-run.
    #[arg(long)]
    dry_run: bool,

    /// How to chunk the payload.
    #[arg(long, default_value_t = 1024)]
    chunk_size: usize,
}

impl Transfer {
    async fn run<S>(
        self, connection: &mut dyn Connection, payload: impl AsRef<Path>, message: &'static str,
        finish: Option<impl FnOnce(Yoke<S::Response<'static>>) -> Result<!>>,
    ) -> Result<()>
    where
        S: for<'a> service::Service<
                Request<'a> = service::transfer::Request<'a>,
                Response<'a> = (),
            >,
    {
        use wasefire_protocol::transfer::Request;
        let Transfer { dry_run, chunk_size } = self;
        let payload = fs::read(payload).await?;
        let progress = indicatif::ProgressBar::new(payload.len() as u64)
            .with_style(
                indicatif::ProgressStyle::with_template(
                    "{msg:9} {elapsed:>3} {spinner} [{wide_bar}] {bytes:>10} / {total_bytes:<10}",
                )?
                .tick_chars("-\\|/ ")
                .progress_chars("##-"),
            )
            .with_message("Starting");
        progress.enable_steady_tick(Duration::from_millis(200));
        connection.call::<S>(Request::Start { dry_run }).await?.get();
        progress.set_message("Writing");
        for chunk in payload.chunks(chunk_size) {
            progress.inc(chunk.len() as u64);
            connection.call::<S>(Request::Write { chunk }).await?.get();
        }
        progress.set_message("Finishing");
        match (dry_run, finish) {
            (false, Some(finish)) => final_call::<S>(connection, Request::Finish, finish).await?,
            _ => *connection.call::<S>(Request::Finish).await?.get(),
        }
        progress.finish_with_message(message);
        Ok(())
    }
}

async fn final_call<S: service::Service>(
    connection: &mut dyn Connection, request: S::Request<'_>,
    proof: impl FnOnce(Yoke<S::Response<'static>>) -> Result<!>,
) -> Result<()> {
    connection.send(&S::request(request)).await?;
    match connection.receive::<S>().await {
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
    pub async fn run(self, connection: &mut dyn Connection) -> Result<()> {
        let PlatformReboot {} = self;
        final_call::<service::PlatformReboot>(connection, (), |x| match *x.get() {}).await
    }
}

/// Locks a platform.
#[derive(clap::Args)]
pub struct PlatformLock {}

impl PlatformLock {
    pub async fn run(self, connection: &mut dyn Connection) -> Result<()> {
        let PlatformLock {} = self;
        connection.call::<service::PlatformLock>(()).await.map(|x| *x.get())
    }
}

/// Calls a vendor RPC on a platform.
#[derive(clap::Args)]
pub struct PlatformRpc {
    #[clap(flatten)]
    rpc: Rpc,
}

impl PlatformRpc {
    pub async fn run(self, connection: &mut dyn Connection) -> Result<()> {
        let PlatformRpc { rpc } = self;
        let request = rpc.read().await?;
        let response = connection.call::<service::PlatformVendor>(&request).await?;
        rpc.write(response.get()).await
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

    /// Builds a native applet, e.g. --native=thumbv7em-none-eabi.
    #[arg(long, value_name = "TARGET")]
    pub native: Option<String>,

    /// Root directory of the crate.
    #[arg(long, value_name = "DIRECTORY", default_value = ".")]
    #[arg(value_hint = ValueHint::DirPath)]
    pub crate_dir: PathBuf,

    /// Copies the final artifacts to this directory.
    #[arg(long, value_name = "DIRECTORY", default_value = "wasefire")]
    #[arg(value_hint = ValueHint::DirPath)]
    pub output_dir: PathBuf,

    /// Cargo profile.
    #[arg(long, default_value = "release")]
    pub profile: String,

    /// Optimization level.
    #[clap(long, short = 'O')]
    pub opt_level: Option<OptLevel>,

    /// Stack size (ignored for native applets).
    #[clap(long, default_value = "16384")]
    pub stack_size: usize,

    /// Extra arguments to cargo, e.g. --features=foo.
    #[clap(last = true)]
    pub cargo: Vec<String>,
}

impl RustAppletBuild {
    pub async fn run(self) -> Result<()> {
        let metadata = metadata(&self.crate_dir).await?;
        let package = &metadata.packages[0];
        let target_dir =
            fs::try_relative(std::env::current_dir()?, &metadata.target_directory).await?;
        let name = package.name.replace('-', "_");
        let mut cargo = Command::new("cargo");
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
            let mut features = "-Zbuild-std-features=panic_immediate_abort".to_string();
            if self.opt_level.is_some_and(OptLevel::optimize_for_size) {
                features.push_str(",optimize_for_size");
            }
            cargo.arg(features);
        } else {
            cargo.env("WASEFIRE_DEBUG", "");
        }
        cargo.env("RUSTFLAGS", rustflags.join(" "));
        cargo.current_dir(&self.crate_dir);
        cmd::execute(&mut cargo).await?;
        let (src, dst) = match &self.native {
            None => (format!("wasm32-unknown-unknown/{profile}/{name}.wasm"), "applet.wasm"),
            Some(target) => (format!("{target}/{profile}/lib{name}.a"), "libapplet.a"),
        };
        let applet = self.output_dir.join(dst);
        if fs::copy_if_changed(target_dir.join(src), &applet).await? && dst.ends_with(".wasm") {
            optimize_wasm(&applet, self.opt_level).await?;
        }
        Ok(())
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
    #[clap(last = true)]
    cargo: Vec<String>,
}

impl RustAppletTest {
    pub async fn run(self) -> Result<()> {
        let metadata = metadata(&self.crate_dir).await?;
        let package = &metadata.packages[0];
        ensure!(package.features.contains_key("test"), "missing test feature");
        let mut cargo = Command::new("cargo");
        cargo.args(["test", "--features=test"]);
        cargo.args(&self.cargo);
        cargo.current_dir(&self.crate_dir);
        cmd::replace(cargo)
    }
}

/// Builds and installs a Rust applet from its project.
#[derive(clap::Parser)]
pub struct RustAppletInstall {
    #[clap(flatten)]
    build: RustAppletBuild,

    #[clap(flatten)]
    transfer: Transfer,

    #[command(subcommand)]
    wait: Option<AppletInstallWait>,
}

impl RustAppletInstall {
    pub async fn run(self, connection: &mut dyn Connection) -> Result<()> {
        let RustAppletInstall { build, transfer, wait } = self;
        let output = build.output_dir.clone();
        build.run().await?;
        let install = AppletInstall { applet: output.join("applet.wasm"), transfer, wait };
        install.run(connection).await
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

/// Strips and optimizes a WASM applet.
pub async fn optimize_wasm(applet: impl AsRef<Path>, opt_level: Option<OptLevel>) -> Result<()> {
    let mut strip = Command::new("wasm-strip");
    strip.arg(applet.as_ref());
    cmd::execute(&mut strip).await?;
    let mut opt = Command::new("wasm-opt");
    opt.args(["--enable-bulk-memory", "--enable-sign-ext", "--enable-mutable-globals"]);
    match opt_level {
        Some(level) => drop(opt.arg(format!("-O{level:#}"))),
        None => drop(opt.arg("-O")),
    }
    opt.arg(applet.as_ref());
    opt.arg("-o");
    opt.arg(applet.as_ref());
    cmd::execute(&mut opt).await?;
    Ok(())
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
