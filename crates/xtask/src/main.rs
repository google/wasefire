// Copyright 2022 Google LLC
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

#![feature(try_blocks)]

use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::error::Error;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use anyhow::{bail, ensure, Context, Result};
use clap::Parser;
use probe_rs::config::TargetSelector;
use probe_rs::{flashing, Permissions, Session};
use rustc_demangle::demangle;
use tokio::process::Command;
use tokio::sync::OnceCell;
use wasefire_cli_tools::{action, cmd, fs};

mod footprint;
mod lazy;
mod textreview;

#[derive(Parser)]
struct Flags {
    #[clap(flatten)]
    options: MainOptions,

    #[clap(subcommand)]
    command: MainCommand,
}

#[test]
fn flags() {
    <Flags as clap::CommandFactory>::command().debug_assert();
}

#[derive(clap::Args)]
struct MainOptions {
    /// Compiles without debugging support.
    #[clap(long)]
    release: bool,

    /// Links the applet as a static library to the platform.
    ///
    /// Requires the `runner` subcommand or the `--native-target` option.
    ///
    /// This option improves performance and footprint but removes the security guarantees provided
    /// by sandboxing the applet using WebAssembly.
    #[clap(long)]
    native: bool,

    /// Specifies the native target triple.
    ///
    /// Must match the runner target if both are provided. This implies `--native`.
    #[clap(long)]
    native_target: Option<String>,

    /// Prints basic size information.
    #[clap(long)]
    size: bool,

    /// Updates footprint.toml with the measured footprint for the provided key.
    ///
    /// The key is a space-separated list of strings.
    #[clap(long)]
    footprint: Option<String>,
}

#[derive(clap::Subcommand)]
enum MainCommand {
    /// Compiles an applet.
    Applet(Applet),

    /// Compiles a runner.
    Runner(Runner),

    /// Waits for an applet to exit.
    WaitApplet(Wait),

    /// Waits for a platform to be ready.
    WaitPlatform(Wait),

    /// Appends a comparison between footprint-base.toml and footprint-pull_request.toml.
    ///
    /// If any file is missing, it is assumed to have no measurements.
    Footprint {
        /// The markdown table is written to this file.
        output: String,
    },

    /// Ensures review can be done in printed form.
    Textreview,
}

#[derive(clap::Args)]
struct Applet {
    #[clap(flatten)]
    options: AppletOptions,

    #[clap(subcommand)]
    command: Option<AppletCommand>,
}

#[derive(Default, clap::Args)]
struct AppletOptions {
    /// Applet language.
    lang: String,

    /// Applet name or path (if starts with dot or slash).
    name: String,

    /// Cargo profile.
    #[clap(long)]
    profile: Option<String>,

    /// Cargo features.
    #[clap(long)]
    features: Vec<String>,

    /// Optimization level.
    #[clap(long, short = 'O')]
    opt_level: Option<action::OptLevel>,

    /// Stack size.
    #[clap(long, default_value = "16384")]
    stack_size: usize,
}

#[derive(clap::Subcommand)]
enum AppletCommand {
    /// Compiles a runner with the applet.
    Runner(Runner),

    /// Installs the applet on a platform.
    Install {
        #[command(flatten)]
        options: action::ConnectionOptions,
        #[command(flatten)]
        action: action::Transfer,
        #[command(subcommand)]
        command: Option<AppletInstallCommand>,
    },
}

#[derive(clap::Subcommand)]
enum AppletInstallCommand {
    /// Waits until the applet exits.
    #[group(id = "AppletInstallCommand::Wait")]
    Wait {
        #[command(flatten)]
        action: action::AppletExitStatus,
    },
}

#[derive(clap::Args)]
struct Runner {
    #[clap(flatten)]
    options: RunnerOptions,

    #[clap(subcommand)]
    command: Option<RunnerCommand>,
}

#[derive(Default, clap::Args)]
struct RunnerOptions {
    /// Runner name.
    name: String,

    /// Platform version.
    ///
    /// How the version string is interpreted is up to the runner. For Nordic, it must be a u32
    /// smaller than u32::MAX. For the host, it must be an hexadecimal byte sequence.
    #[clap(long)]
    version: Option<String>,

    /// Cargo no-default-features.
    #[clap(long)]
    no_default_features: bool,

    /// Cargo features.
    #[clap(long)]
    features: Vec<String>,

    /// Optimization level (0, 1, 2, 3, s, z).
    #[clap(long, short = 'O')]
    opt_level: Option<action::OptLevel>,

    /// Prints the command lines to use GDB.
    #[clap(long)]
    gdb: bool,

    /// Defmt log filter.
    #[clap(long)]
    log: Option<String>,

    /// Creates a web interface for the host runner.
    #[clap(long)]
    web: bool,

    /// Host to start the webserver.
    #[clap(long)]
    web_host: Option<String>,

    /// Port to start the webserver.
    #[clap(long)]
    web_port: Option<u16>,

    /// Measures bloat after building.
    // TODO: Make this a subcommand taking additional options for cargo bloat.
    #[clap(long)]
    measure_bloat: bool,

    /// Show the (top N) stack sizes of the firmware.
    #[clap(long)]
    stack_sizes: Option<Option<usize>>,

    /// Allocates <MEMORY_PAGE_COUNT> pages for the WASM module.
    ///
    /// Supported values are numbers between 0 and 9 inclusive, i.e. single digit. The default when
    /// missing is 1 page.
    #[clap(long)]
    memory_page_count: Option<usize>,
}

#[derive(clap::Subcommand)]
enum RunnerCommand {
    /// Flashes the runner.
    Flash(Flash),

    /// Produces target/wasefire/platform_{side}.bin files instead of flashing.
    Bundle,
}

#[derive(clap::Args)]
struct Flash {
    /// Resets the flash before running.
    #[clap(long)]
    reset_flash: bool,

    /// Additional flags for `probe-rs run`.
    #[clap(long)]
    probe_rs: Vec<String>,
}

#[derive(clap::Args)]
struct Wait {
    #[command(flatten)]
    options: action::ConnectionOptions,
}

impl Flags {
    async fn execute(self) -> Result<()> {
        match self.command {
            MainCommand::Applet(applet) => applet.execute(&self.options).await,
            MainCommand::Runner(runner) => runner.execute(&self.options).await,
            MainCommand::WaitApplet(wait) => wait.execute(true).await,
            MainCommand::WaitPlatform(wait) => wait.execute(false).await,
            MainCommand::Footprint { output } => footprint::compare(&output).await,
            MainCommand::Textreview => textreview::execute().await,
        }
    }
}

impl MainOptions {
    fn is_native(&self) -> bool {
        self.native || self.native_target.is_some()
    }
}

impl Applet {
    async fn execute(self, main: &MainOptions) -> Result<()> {
        self.options.execute(main, &self.command).await?;
        if let Some(command) = self.command {
            command.execute(main).await?;
        }
        Ok(())
    }
}

impl AppletOptions {
    async fn execute(self, main: &MainOptions, command: &Option<AppletCommand>) -> Result<()> {
        if !main.is_native() {
            ensure_command(&["wasm-strip"]).await?;
            ensure_command(&["wasm-opt"]).await?;
        }
        match self.lang.as_str() {
            "rust" => self.execute_rust(main, command).await,
            "assemblyscript" => self.execute_assemblyscript(main).await,
            x => bail!("unsupported language {x}"),
        }
    }

    async fn execute_rust(self, main: &MainOptions, command: &Option<AppletCommand>) -> Result<()> {
        let dir = if self.name.starts_with(['.', '/']) {
            self.name.clone()
        } else {
            format!("examples/{}/{}", self.lang, self.name)
        };
        ensure!(fs::exists(&dir).await, "{dir} does not exist");
        let native = match (main.native, &main.native_target, command) {
            (_, Some(target), command) => {
                if let Some(AppletCommand::Runner(x)) = command {
                    ensure!(
                        target == x.options.target().await,
                        "--native-target must match runner"
                    );
                }
                Some(target.as_str())
            }
            (true, None, Some(AppletCommand::Runner(x))) => Some(x.options.target().await),
            (true, None, _) => bail!("--native requires runner"),
            (false, _, _) => None,
        };
        let mut action = action::RustAppletBuild {
            prod: main.release,
            native: native.map(|x| x.to_string()),
            profile: self.profile.clone(),
            opt_level: self.opt_level,
            stack_size: self.stack_size,
            ..Default::default()
        };
        for features in &self.features {
            action.cargo.push(format!("--features={features}"));
        }
        action.run(dir).await?;
        if !main.size && main.footprint.is_none() {
            return Ok(());
        }
        let size = match native {
            Some(_) => footprint::rust_size("target/wasefire/libapplet.a").await?,
            None => fs::metadata("target/wasefire/applet.wasm").await?.len() as usize,
        };
        if main.size {
            println!("Size: {size}");
        }
        if let Some(key) = &main.footprint {
            footprint::update_applet(key, size).await?;
        }
        Ok(())
    }

    async fn execute_assemblyscript(&self, main: &MainOptions) -> Result<()> {
        ensure!(!main.is_native(), "native applets are not supported for assemblyscript");
        let dir = format!("examples/{}", self.lang);
        ensure_assemblyscript().await?;
        let mut asc = Command::new("./node_modules/.bin/asc");
        asc.args(["-o", "../../target/wasefire/applet.wasm"]);
        match self.opt_level {
            Some(level) => drop(asc.arg(format!("-O{level}"))),
            None => drop(asc.arg("-O")),
        }
        asc.args(["--lowMemoryLimit", "--stackSize", &format!("{}", self.stack_size)]);
        asc.args(["--use", &format!("abort={}/main/abort", self.name)]);
        if main.release {
            asc.arg("--noAssert");
        } else {
            asc.arg("--debug");
        }
        asc.arg(format!("{}/main.ts", self.name));
        asc.current_dir(dir);
        cmd::execute(&mut asc).await?;
        action::optimize_wasm("target/wasefire/applet.wasm", self.opt_level).await?;
        Ok(())
    }
}

impl AppletCommand {
    async fn execute(self, main: &MainOptions) -> Result<()> {
        match self {
            AppletCommand::Runner(runner) => runner.execute(main).await,
            AppletCommand::Install { options, action, command } => {
                let applet = "target/wasefire/applet.wasm".into();
                let action = action::AppletInstall { applet, transfer: action };
                let mut connection = options.connect().await?;
                action.run(&mut connection).await?;
                match command {
                    None => Ok(()),
                    Some(AppletInstallCommand::Wait { mut action }) => {
                        action.wait.ensure_wait();
                        action.ensure_exit();
                        action.run(&mut connection).await
                    }
                }
            }
        }
    }
}

impl Runner {
    async fn execute(&self, main: &MainOptions) -> Result<()> {
        self.options.execute(main, 0, &self.command).await?;
        Ok(())
    }
}

impl RunnerOptions {
    async fn execute(
        &self, main: &MainOptions, step: usize, cmd: &Option<RunnerCommand>,
    ) -> Result<()> {
        let flash = match cmd {
            Some(RunnerCommand::Flash(x)) => Some(x),
            Some(RunnerCommand::Bundle) => None,
            None => None,
        };
        let mut cargo = Command::new("cargo");
        let mut rustflags = Vec::new();
        let mut features = self.features.clone();
        if flash.is_some() && self.name == "host" {
            cargo.arg("run");
        } else {
            cargo.arg("build");
        }
        cargo.arg("--release");
        cargo.arg(format!("--target={}", self.target().await));
        let (side, max_step) = match self.name.as_str() {
            "nordic" => (Some(step), 1),
            "host" => (None, 0),
            _ => unimplemented!(),
        };
        let side = match side {
            None => "",
            Some(0) => "_a",
            Some(1) => "_b",
            _ => unimplemented!(),
        };
        if self.name == "host" {
            if let Some(version) = &self.version {
                cargo.env("WASEFIRE_HOST_VERSION", version);
            };
        }
        if self.name == "nordic" {
            rustflags.push(format!("-C link-arg=--defsym=RUNNER_SIDE={step}"));
            let version = match &self.version {
                Some(x) => x.parse()?,
                None => 0,
            };
            ensure!(version < u32::MAX, "--version must be smaller than u32::MAX");
            rustflags.push(format!("-C link-arg=--defsym=RUNNER_VERSION={version}"));
            rustflags.push("-C link-arg=-Tlink.x".to_string());
            if main.release {
                cargo.arg("-Zbuild-std=core,alloc");
                let mut features = "-Zbuild-std-features=panic_immediate_abort".to_string();
                if self.opt_level.map_or(false, action::OptLevel::optimize_for_size) {
                    features.push_str(",optimize_for_size");
                }
                cargo.arg(features);
                cargo.arg("--config=profile.release.codegen-units=1");
                cargo.arg("--config=profile.release.lto=true");
            } else {
                cargo.arg("--config=profile.release.debug=2");
                rustflags.push("-C link-arg=-Tdefmt.x".to_string());
            }
        }
        if let Some(level) = self.opt_level {
            cargo.arg(format!("--config=profile.release.opt-level={level}"));
        }
        if main.release {
            features.push("release".to_string());
        } else {
            features.push("debug".to_string());
        }
        if self.no_default_features {
            cargo.arg("--no-default-features");
        } else if std::env::var_os("CODESPACES").is_some() {
            log::warn!("Assuming runner --no-default-features when running in a codespace.");
            cargo.arg("--no-default-features");
        }
        if let Some(log) = &self.log {
            cargo.env(self.log_env(), log);
        }
        let web = self.web || self.web_host.is_some() || self.web_port.is_some();
        if self.name == "host" && web {
            features.push("web".to_string());
        }
        if self.stack_sizes.is_some() {
            rustflags.push("-Z emit-stack-sizes".to_string());
            rustflags.push("-C link-arg=-Tstack-sizes.x".to_string());
        }
        if main.native {
            features.push("native".to_string());
        } else {
            features.push("wasm".to_string());
        }
        if !features.is_empty() {
            cargo.arg(format!("--features={}", features.join(",")));
        }
        if let Some(n) = self.memory_page_count {
            ensure!((0 ..= 9).contains(&n), "--memory-page-count supports single digit only");
            cargo.env("WASEFIRE_MEMORY_PAGE_COUNT", format!("{n}"));
        }
        if !rustflags.is_empty() {
            cargo.env("RUSTFLAGS", rustflags.join(" "));
        }
        cargo.current_dir(format!("crates/runner-{}", self.name));
        if flash.is_some() && self.name == "host" {
            if flash.unwrap().reset_flash {
                for file in ["applet.bin", "storage.bin"] {
                    let path = format!("target/wasefire/{file}");
                    if fs::exists(&path).await {
                        fs::remove_file(&path).await?;
                    }
                }
            }
            cargo.arg("--");
            if let Some(host) = &self.web_host {
                cargo.arg(format!("--web-host={host}"));
            }
            if let Some(port) = &self.web_port {
                cargo.arg(format!("--web-port={port}"));
            }
            cmd::replace(cargo);
        } else {
            cmd::execute(&mut cargo).await?;
        }
        if self.measure_bloat {
            ensure_command(&["cargo", "bloat"]).await?;
            let mut bloat = wrap_command().await?;
            bloat.arg(cargo.as_std().get_program());
            if let Some(dir) = cargo.as_std().get_current_dir() {
                bloat.current_dir(dir);
            }
            for (key, val) in cargo.as_std().get_envs() {
                match val {
                    None => bloat.env_remove(key),
                    Some(val) => bloat.env(key, val),
                };
            }
            for arg in cargo.as_std().get_args() {
                if arg == "build" {
                    bloat.arg("bloat");
                } else {
                    bloat.arg(arg);
                }
            }
            bloat.args(["--crates", "--split-std"]);
            cmd::execute(&mut bloat).await?;
        }
        let elf = self.board_target().await;
        if main.size {
            let mut size = wrap_command().await?;
            size.arg("rust-size");
            size.arg(&elf);
            cmd::execute(&mut size).await?;
        }
        if let Some(key) = &main.footprint {
            footprint::update_runner(key, footprint::rust_size(&elf).await?).await?;
        }
        if let Some(stack_sizes) = self.stack_sizes {
            let elf = fs::read(&elf).await?;
            let symbols = stack_sizes::analyze_executable(&elf)?;
            assert!(symbols.have_32_bit_addresses);
            assert!(symbols.undefined.is_empty());
            let max_stack_sizes = stack_sizes.unwrap_or(10);
            let mut top_stack_sizes = BinaryHeap::new();
            for (address, symbol) in symbols.defined {
                let stack = match symbol.stack() {
                    None => continue,
                    Some(x) => x,
                };
                // Multiple symbols can have the same address. Just use the first name.
                let name = *symbol.names().first().context("missing symbol")?;
                top_stack_sizes.push((Reverse(stack), address, name));
                if top_stack_sizes.len() > max_stack_sizes {
                    top_stack_sizes.pop();
                }
            }
            while let Some((Reverse(stack), address, name)) = top_stack_sizes.pop() {
                println!("{:#010x}\t{}\t{}", address, stack, demangle(name));
            }
        }
        if matches!(cmd, Some(RunnerCommand::Bundle)) {
            let mut objcopy = wrap_command().await?;
            objcopy.args(["rust-objcopy", "-O", "binary", &elf]);
            objcopy.arg(format!("target/wasefire/platform{side}.bin"));
            cmd::execute(&mut objcopy).await?;
            if step < max_step {
                return Box::pin(self.execute(main, step + 1, cmd)).await;
            }
            return Ok(());
        }
        let flash = match flash {
            Some(x) => x,
            None => return Ok(()),
        };
        let chip = match self.name.as_str() {
            "nordic" => "nRF52840_xxAA",
            "host" => unreachable!(),
            _ => unimplemented!(),
        };
        let session = Arc::new(Mutex::new(lazy::Lazy::new(|| {
            Ok(Session::auto_attach(
                TargetSelector::Unspecified(chip.to_string()),
                Permissions::default(),
            )?)
        })));
        if flash.reset_flash {
            println!("Erasing the flash.");
            // Keep those values in sync with crates/runner-nordic/memory.x.
            tokio::task::spawn_blocking({
                let session = session.clone();
                move || {
                    let mut session = session.lock().unwrap();
                    anyhow::Ok(flashing::erase_all(session.get()?, None)?)
                }
            })
            .await??;
        }
        if self.name == "nordic" {
            let mut cargo = Command::new("cargo");
            cargo.current_dir("crates/runner-nordic/crates/bootloader");
            cargo.args(["build", "--release", "--target=thumbv7em-none-eabi"]);
            cargo.args(["-Zbuild-std=core", "-Zbuild-std-features=panic_immediate_abort"]);
            cmd::execute(&mut cargo).await?;
            tokio::task::spawn_blocking(move || {
                anyhow::Ok(flashing::download_file(
                    session.lock().unwrap().get()?,
                    "target/thumbv7em-none-eabi/release/bootloader",
                    flashing::Format::Elf,
                )?)
            })
            .await??;
        }
        if self.gdb {
            println!("Use the following 2 commands in different terminals:");
            println!("JLinkGDBServer -device {chip} -if swd -speed 4000 -port 2331");
            println!("gdb-multiarch -ex 'file {elf}' -ex 'target remote localhost:2331'");
        }
        let mut probe_rs = wrap_command().await?;
        probe_rs.args(["probe-rs", "run"]);
        probe_rs.arg(format!("--chip={chip}"));
        probe_rs.args(&flash.probe_rs);
        probe_rs.arg(elf);
        println!("Replace `run` with `attach` in the following command to rerun:");
        cmd::replace(probe_rs);
    }

    async fn target(&self) -> &'static str {
        // Each time we specify RUSTFLAGS, we want to specify --target. This is because if --target
        // is not specified then RUSTFLAGS applies to all compiler invocations (including build
        // scripts and proc macros). This leads to recompilation when RUSTFLAGS changes. See
        // https://github.com/rust-lang/cargo/issues/8716.
        static HOST_TARGET: OnceCell<String> = OnceCell::const_new();
        match self.name.as_str() {
            "nordic" => "thumbv7em-none-eabi",
            "host" => {
                HOST_TARGET
                    .get_or_init(|| async {
                        let mut sh = Command::new("sh");
                        sh.args(["-c", "rustc -vV | sed -n 's/^host: //p'"]);
                        cmd::output_line(&mut sh).await.unwrap()
                    })
                    .await
            }
            _ => unimplemented!(),
        }
    }

    fn log_env(&self) -> &'static str {
        match self.name.as_str() {
            "nordic" => "DEFMT_LOG",
            "host" => "RUST_LOG",
            _ => unimplemented!(),
        }
    }

    async fn board_target(&self) -> String {
        format!("target/{}/release/runner-{}", self.target().await, self.name)
    }
}

impl Wait {
    async fn execute(&self, applet: bool) -> Result<()> {
        let period = Duration::from_millis(300);
        loop {
            tokio::time::sleep(period).await;
            let mut action = action::AppletExitStatus::default();
            action.wait.set_period(period);
            if applet {
                action.ensure_exit();
            }
            let mut connection = match self.options.connect().await {
                Ok(x) => x,
                Err(_) => continue,
            };
            let error = match action.run(&mut connection).await {
                Err(x) => x,
                Ok(_) => continue,
            };
            use wasefire_error::{Code, Error};
            if root_cause_is::<Error>(&error, |&x| x == Error::user(Code::NotFound)) {
                break Ok(());
            }
        }
    }
}

fn root_cause_is<E: Error + Send + Sync + 'static>(
    error: &anyhow::Error, predicate: impl FnOnce(&E) -> bool,
) -> bool {
    error.root_cause().downcast_ref::<E>().map_or(false, predicate)
}

async fn ensure_command(cmd: &[&str]) -> Result<()> {
    let mut wrapper = Command::new("./scripts/wrapper.sh");
    wrapper.args(cmd);
    wrapper.env("WASEFIRE_WRAPPER_EXEC", "n");
    cmd::execute(&mut wrapper).await
}

async fn wrap_command() -> Result<Command> {
    Ok(Command::new(fs::canonicalize("./scripts/wrapper.sh").await?))
}

async fn ensure_assemblyscript() -> Result<()> {
    const ASC_VERSION: &str = "0.27.29"; // scripts/upgrade.sh relies on this name
    const BIN: &str = "examples/assemblyscript/node_modules/.bin/asc";
    const JSON: &str = "examples/assemblyscript/node_modules/assemblyscript/package.json";
    if fs::exists(BIN).await && fs::exists(JSON).await {
        let mut sed = Command::new("sed");
        sed.args(["-n", r#"s/^  "version": "\(.*\)",$/\1/p"#, JSON]);
        if cmd::output_line(&mut sed).await? == ASC_VERSION {
            return Ok(());
        }
    }
    ensure_command(&["npm"]).await?;
    let mut npm = wrap_command().await?;
    npm.args(["npm", "install", "--no-save"]);
    npm.arg(format!("assemblyscript@{ASC_VERSION}"));
    npm.current_dir("examples/assemblyscript");
    cmd::execute(&mut npm).await
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("warn"));
    Flags::parse().execute().await?;
    Ok(())
}
