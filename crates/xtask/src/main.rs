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

#![feature(never_type)]
#![feature(try_blocks)]

use std::borrow::Cow;
use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::ffi::OsString;
use std::path::PathBuf;
use std::time::Duration;

use anyhow::{Context, Result, anyhow, bail, ensure};
use clap::{Parser, ValueEnum};
use data_encoding::HEXLOWER_PERMISSIVE as HEX;
use object::read::elf::{ElfFile32, SectionHeader as _};
use rustc_demangle::demangle;
use tokio::process::Command;
use tokio::sync::OnceCell;
use wasefire_cli_tools::error::root_cause_is;
use wasefire_cli_tools::{action, changelog, cmd, fs};

mod footprint;
mod opentitan;
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

    /// Compiles applets to Pulley and platforms with a Pulley interpreter.
    ///
    /// This experimental option improves performance for footprint.
    #[clap(long, conflicts_with_all = ["native", "native_target"])]
    pulley: bool,

    /// Prints basic size information.
    #[clap(long)]
    size: bool,

    /// Updates footprint.toml with the measured footprint for the provided key.
    ///
    /// The key is a space-separated list of strings.
    #[clap(long)]
    footprint: Option<String>,

    /// Whether to run setsid before spawning processes.
    #[clap(long)]
    setsid: bool,
}

#[derive(clap::Subcommand)]
enum MainCommand {
    /// Compiles an applet.
    Applet(Applet),

    /// Compiles a runner.
    Runner(Runner),

    /// Attaches to a runner.
    Attach(Attach),

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

    /// Performs a changelog operation.
    Changelog(Changelog),
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
        transfer: action::Transfer,
        #[command(subcommand)]
        wait: Option<action::AppletInstallWait>,
    },
}

#[derive(Copy, Clone, PartialEq, Eq, ValueEnum)]
enum RunnerName {
    #[value(name = "host")]
    Host,
    #[value(name = "nordic")]
    Nordic,
    #[value(name = "opentitan")]
    OpenTitan,
}

impl std::fmt::Display for RunnerName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            RunnerName::Host => "host",
            RunnerName::Nordic => "nordic",
            RunnerName::OpenTitan => "opentitan",
        };
        write!(f, "{name}")
    }
}

impl RunnerName {
    fn default_board(self) -> Option<&'static str> {
        match self {
            RunnerName::Host => None,
            RunnerName::Nordic => Some("devkit"),
            RunnerName::OpenTitan => None,
        }
    }

    fn chip(self) -> &'static str {
        match self {
            RunnerName::Host => unreachable!(),
            RunnerName::Nordic => "nRF52840_xxAA",
            RunnerName::OpenTitan => unreachable!(),
        }
    }

    fn log_env(self) -> &'static str {
        match self {
            RunnerName::Host => "RUST_LOG",
            RunnerName::Nordic => "DEFMT_LOG",
            RunnerName::OpenTitan => "DEFMT_LOG",
        }
    }

    async fn target(self, main: &MainOptions) -> &'static str {
        // Each time we specify RUSTFLAGS, we want to specify --target. This is because if --target
        // is not specified then RUSTFLAGS applies to all compiler invocations (including build
        // scripts and proc macros). This leads to recompilation when RUSTFLAGS changes. See
        // https://github.com/rust-lang/cargo/issues/8716.
        static HOST_TARGET: OnceCell<String> = OnceCell::const_new();
        match self {
            RunnerName::Host if main.is_native() || main.pulley => "i686-unknown-linux-gnu",
            RunnerName::Host => {
                HOST_TARGET
                    .get_or_init(|| async {
                        cmd::output_line(Command::new("rustc").args(["--print", "host-tuple"]))
                            .await
                            .unwrap()
                    })
                    .await
            }
            RunnerName::Nordic => "thumbv7em-none-eabi",
            RunnerName::OpenTitan => "riscv32imc-unknown-none-elf",
        }
    }

    async fn elf(self, main: &MainOptions) -> String {
        format!("target/{}/release/runner-{}", self.target(main).await, self)
    }
}

#[derive(clap::Args)]
struct Runner {
    #[clap(flatten)]
    options: RunnerOptions,

    #[clap(subcommand)]
    command: Option<RunnerCommand>,
}

#[derive(clap::Args)]
struct RunnerOptions {
    /// Runner name.
    name: RunnerName,

    /// Platform name (in hexadecimal form).
    ///
    /// Each runner has its own length constraints:
    /// - Host supports any number of bytes.
    /// - Nordic supports 24 bytes.
    /// - OpenTitan supports 0 bytes.
    ///
    /// The default is the shortest name made of null bytes only.
    #[clap(long, verbatim_doc_comment)]
    name_hex: Option<String>,

    /// Platform name (in string form).
    ///
    /// See --name-hex for the maximum length. Characters should be ASCII and graphic.
    #[clap(long, conflicts_with("name_hex"))]
    name_str: Option<String>,

    /// Platform version (big-endian hexadecimal number).
    ///
    /// Each runner has its own format:
    /// - Host supports any hexadecimal string.
    /// - Nordic needs 4 bytes that are not all 0xff.
    /// - OpenTitan needs 20 bytes that are not all 0xff. The bytes are the major version (4
    ///   bytes), the minor version (4 bytes), the security version (4 bytes), and the timestamp (8
    ///   bytes).
    #[clap(long, verbatim_doc_comment)]
    version: Option<String>,

    /// Host platform serial.
    ///
    /// This is only used for the host runner. It must be an hexadecimal byte sequence.
    #[clap(long)]
    serial: Option<String>,

    /// Cargo no-default-features.
    #[clap(long)]
    no_default_features: bool,

    /// Cargo features.
    #[clap(long)]
    features: Vec<String>,

    /// Optimization level (0, 1, 2, 3, s, z).
    #[clap(long, short = 'O')]
    opt_level: Option<action::OptLevel>,

    /// Board selection.
    ///
    /// Each runner supports its own set of boards:
    /// - Host doesn't have a notion of board.
    /// - Nordic supports devkit (default), dongle, and makerdiary.
    /// - OpenTitan doesn't have a notion of board yet.
    #[clap(long, verbatim_doc_comment)]
    board: Option<String>,

    /// Disables A/B platform partitioning.
    ///
    /// This is only supported on Nordic. This can be useful for debugging when a platform is too
    /// large. As a consequence, it is not possible to update such a platform.
    #[clap(long)]
    single_sided: bool,

    /// Prints the command lines to use GDB.
    #[clap(long)]
    gdb: bool,

    /// Defmt or log filter.
    #[clap(long)]
    log: Option<String>,

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
    /// Updates the runner.
    Update(Update),

    /// Flashes the runner (after erasing the device).
    Flash(Flash),

    /// Produces the target/wasefire/platform{,_a,_b}.bin files for platform update.
    Bundle {
        #[clap(skip)]
        step: usize,
    },
}

#[derive(clap::Args)]
struct Update {
    #[command(flatten)]
    options: action::ConnectionOptions,
    #[command(flatten)]
    transfer: action::Transfer,

    /// Updates both sides of the runner.
    #[clap(long)]
    both: bool,

    #[clap(skip)]
    step: Option<usize>,

    #[clap(subcommand)]
    command: Option<UpdateCommand>,
}

#[derive(clap::Subcommand)]
enum UpdateCommand {
    /// Attaches to the runner after the update.
    Attach(AttachOptions),
}

#[derive(clap::Args)]
struct Flash {
    /// Only produce artifacts and do not flash them.
    ///
    /// This is different from the bundle command which produces artifacts for the purpose of
    /// updating the platform. The artifacts here are meant for flashing the platform.
    #[clap(long)]
    artifacts: bool,

    #[clap(flatten)]
    attach: AttachOptions,
}

#[derive(clap::Args)]
struct Attach {
    /// Runner name.
    name: RunnerName,

    /// Log filter for Host runner.
    ///
    /// This is for host only, because the defmt filter is used at compile-time.
    #[clap(long)]
    log: Option<String>,

    #[clap(flatten)]
    options: AttachOptions,
}

#[derive(Default, clap::Args)]
struct AttachOptions {
    /// Additional flags for `defmt-print run`.
    ///
    /// This is only for the OpenTitan runner so far.
    #[clap(long)]
    defmt_print: Vec<String>,

    /// Arguments to forward to the runner.
    ///
    /// This can be `probe-rs run` for non-host runners.
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    args: Vec<String>,
}

#[derive(clap::Args)]
struct Wait {
    #[command(flatten)]
    options: action::ConnectionOptions,
}

#[derive(clap::Args)]
struct Changelog {
    #[clap(subcommand)]
    command: ChangelogCommand,
}

#[derive(clap::Subcommand)]
enum ChangelogCommand {
    /// Validates all changelogs.
    Ci,

    /// Logs a crate change.
    Change {
        /// Path to the changed crate.
        path: String,

        /// Severity of the change.
        severity: changelog::Severity,

        /// One-line description of the change.
        description: String,
    },
}

impl Flags {
    async fn execute(self) -> Result<()> {
        if self.options.setsid {
            unsafe { libc::setsid() };
        }
        match self.command {
            MainCommand::Applet(applet) => applet.execute(&self.options).await,
            MainCommand::Runner(runner) => runner.execute(&self.options).await,
            MainCommand::Attach(attach) => attach.execute(&self.options).await?,
            MainCommand::WaitApplet(wait) => wait.execute(true).await,
            MainCommand::WaitPlatform(wait) => wait.execute(false).await,
            MainCommand::Footprint { output } => footprint::compare(&output).await,
            MainCommand::Textreview => textreview::execute().await,
            MainCommand::Changelog(subcommand) => match subcommand.command {
                ChangelogCommand::Ci => changelog::execute_ci().await,
                ChangelogCommand::Change { path, severity, description } => {
                    changelog::execute_change(&path, &severity, &description).await
                }
            },
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
        let dir: PathBuf = if self.name.starts_with(['.', '/']) {
            self.name.into()
        } else {
            ["examples", &self.lang, &self.name].into_iter().collect()
        };
        ensure!(fs::exists(&dir).await, "{} does not exist", dir.display());
        let native = match (main.native, &main.native_target, command) {
            (_, Some(target), command) => {
                if let Some(AppletCommand::Runner(x)) = command {
                    ensure!(
                        target == x.options.name.target(main).await,
                        "--native-target must match runner"
                    );
                }
                Some(target.as_str())
            }
            (true, None, Some(AppletCommand::Runner(x))) => Some(x.options.name.target(main).await),
            (true, None, _) => bail!("--native requires runner"),
            (false, _, _) => None,
        };
        let mut action = action::RustAppletBuild {
            prod: main.release,
            native: native.map(|x| x.to_string()),
            pulley: main.pulley,
            opt_level: self.opt_level,
            stack_size: self.stack_size,
            crate_dir: dir,
            output_dir: "target/wasefire".into(),
            ..action::RustAppletBuild::parse_from::<_, OsString>([])
        };
        if let Some(profile) = self.profile {
            action.profile = profile;
        }
        for features in &self.features {
            action.cargo.push(format!("--features={features}"));
        }
        action.run().await?;
        if !main.size && main.footprint.is_none() {
            return Ok(());
        }
        let size = match native {
            Some(_) => footprint::rust_size("target/wasefire/libapplet.a").await?,
            None if main.pulley => {
                fs::metadata("target/wasefire/applet.pulley").await?.len() as usize
            }
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
        let src = "target/wasefire/applet-assemblyscript.wasm";
        asc.args(["-o", &format!("../../{src}")]);
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
        let opt = "target/wasefire/applet-opt.wasm";
        action::optimize_wasm(src, self.opt_level, opt).await?;
        if main.pulley {
            let dst = "target/wasefire/applet.pulley";
            action::compile_pulley(opt, dst).await?;
        } else {
            let dst = "target/wasefire/applet.wasm";
            action::compute_sidetable(opt, dst).await?;
        }
        Ok(())
    }
}

impl AppletCommand {
    async fn execute(self, main: &MainOptions) -> Result<()> {
        match self {
            AppletCommand::Runner(runner) => runner.execute(main).await,
            AppletCommand::Install { options, transfer, mut wait } => {
                let applet = if main.pulley {
                    "target/wasefire/applet.pulley".into()
                } else {
                    "target/wasefire/applet.wasm".into()
                };
                if let Some(action::AppletInstallWait::Wait { action }) = &mut wait {
                    action.ensure_exit();
                }
                let action = action::AppletInstall { applet, transfer, wait };
                action.run(&options.connect().await?).await
            }
        }
    }
}

impl Runner {
    async fn execute(self, main: &MainOptions) -> Result<()> {
        self.options.execute(main, self.command).await?;
        Ok(())
    }
}

impl RunnerOptions {
    async fn execute(mut self, main: &MainOptions, cmd: Option<RunnerCommand>) -> Result<()> {
        let mut version = self.version.as_deref().map(Cow::Borrowed);
        ensure!(
            !self.single_sided || matches!(cmd, None | Some(RunnerCommand::Flash(_))),
            "unsupported runner command for single-sided platforms"
        );
        let mut step = match &cmd {
            Some(RunnerCommand::Bundle { step }) => *step,
            _ => 0,
        };
        let flash = match &cmd {
            Some(RunnerCommand::Flash(x)) => Some(x),
            _ => None,
        };
        if flash.is_some_and(|x| x.artifacts) {
            match self.name {
                RunnerName::Host => bail!("--artifacts is not supported for host"),
                RunnerName::Nordic => (),
                x => bail!("--artifacts is not implemented for {x}"),
            }
        }
        let update = match &cmd {
            Some(RunnerCommand::Update(Update { options, both, step: next_step, .. })) => {
                ensure!(
                    !both || options.reboot_stable(),
                    "--both requires a reboot-stable protocol"
                );
                let device = match next_step {
                    None => options.connect().await?,
                    Some(_) => loop {
                        tokio::time::sleep(Duration::from_millis(300)).await;
                        if let Ok(x) = options.connect().await {
                            break x;
                        }
                    },
                };
                let info = device.platform_info().await?;
                let Some(running_side) = info.running_side() else {
                    bail!("device does not support platform update")
                };
                match next_step {
                    None => {
                        if version.is_none() {
                            let mut next_version = info.running_version().to_vec();
                            for byte in next_version.iter_mut().rev() {
                                *byte = byte.wrapping_add(1);
                                if 0 < *byte {
                                    break;
                                }
                            }
                            version = Some(Cow::Owned(HEX.encode(&next_version)));
                        }
                        step = running_side.opposite() as usize;
                    }
                    Some(next_step) => {
                        let version = HEX.decode(version.as_ref().unwrap().as_bytes())?;
                        step = *next_step;
                        ensure!(info.running_version() == version, "first update failed");
                        ensure!(running_side as usize == 1 - step, "first update failed");
                    }
                }
                Some(device)
            }
            _ => None,
        };
        let mut cargo = Command::new("cargo");
        let mut rustflags = Vec::new();
        let mut features = self.features.clone();
        if flash.is_some() && self.name == RunnerName::Host {
            cargo.arg("run");
        } else {
            cargo.arg("build");
        }
        cargo.arg("--release");
        cargo.arg(format!("--target={}", self.name.target(main).await));
        let (side, max_step) = match self.name {
            RunnerName::Host => (None, 0),
            RunnerName::Nordic | RunnerName::OpenTitan => (Some(step), 1),
        };
        if self.name == RunnerName::Host {
            if std::env::var_os("PKG_CONFIG_SYSROOT_DIR").is_none() {
                cargo.env("PKG_CONFIG_SYSROOT_DIR", "/usr/lib/i386-linux-gnu/pkgconfig");
            }
            if let Some(name_hex) = &self.name_hex {
                cargo.env("WASEFIRE_HOST_NAME", name_hex);
            }
            if let Some(name_str) = &self.name_str {
                cargo.env("WASEFIRE_HOST_NAME", HEX.encode(name_str.as_bytes()));
            }
            if let Some(version) = version.as_deref() {
                cargo.env("WASEFIRE_HOST_VERSION", version);
            }
            if let Some(serial) = &self.serial {
                cargo.env("WASEFIRE_HOST_SERIAL", serial);
            }
            if fs::exists("target/wasefire/web").await {
                fs::remove_dir_all("target/wasefire/web").await?;
            }
            cmd::execute(Command::new("make").current_dir("crates/runner-host/crates/web-client"))
                .await?;
        } else {
            let native = main.is_native() as u8;
            rustflags.push(format!("-C link-arg=--defsym=RUNNER_NATIVE={native}"));
            rustflags.push(format!("-C link-arg=--defsym=RUNNER_SIDE={step}"));
            if self.name == RunnerName::Nordic {
                let num_sides = if self.single_sided { 1 } else { 2 };
                rustflags.push(format!("-C link-arg=--defsym=RUNNER_NUM_SIDES={num_sides}"));
            }
            if self.name == RunnerName::OpenTitan {
                const OPENTITAN: &str = "third_party/lowRISC/opentitan";
                const CRYPTODIR: &str = "sw/device/lib/crypto";
                let mut bazel = wrap_command().await?;
                bazel.current_dir(OPENTITAN);
                bazel.args(["bazel", "build", &format!("//{CRYPTODIR}:otcrypto")]);
                cmd::execute(&mut bazel).await?;
                rustflags.push(format!("-C link-arg=-L../../{OPENTITAN}/bazel-bin/{CRYPTODIR}"));
                rustflags.push("-C link-arg=-lotcrypto".to_string());
                rustflags.push("-C link-arg=-Tmemory.x".to_string());
                cargo.env("RISCV_MTVEC_ALIGN", "256");
            }
            rustflags.push("-C link-arg=-Tlink.x".to_string());
            if main.release {
                cargo.arg("-Zbuild-std=core,alloc");
                // TODO(https://github.com/rust-lang/rust/issues/122105): Remove when fixed.
                rustflags.push("--allow=unused-crate-dependencies".to_string());
                if self.opt_level.is_some_and(action::OptLevel::optimize_for_size) {
                    cargo.arg("-Zbuild-std-features=optimize_for_size");
                }
                cargo.arg("--config=profile.release.codegen-units=1");
                cargo.arg("--config=profile.release.lto=true");
                cargo.arg("--config=profile.release.panic=\"immediate-abort\"");
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
        }
        if let Some(log) = &self.log {
            cargo.env(self.name.log_env(), log);
        }
        if self.stack_sizes.is_some() {
            rustflags.push("-Z emit-stack-sizes".to_string());
            rustflags.push("-C link-arg=-Tstack-sizes.x".to_string());
        }
        if main.is_native() {
            features.push("native".to_string());
        } else if main.pulley {
            features.push("pulley".to_string());
            cargo.arg("--config=profile.release.package.wasmtime.opt-level=\"z\"");
            cargo.arg("--config=profile.release.package.pulley-interpreter.opt-level=3");
        } else {
            features.push("wasm".to_string());
            cargo.arg("--config=profile.release.package.wasefire-interpreter.opt-level=3");
        }
        let board = self.name.default_board().map(|x| self.board.as_deref().unwrap_or(x));
        if let Some(board) = board {
            features.push(format!("board-{board}"));
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
        if flash.is_some() && self.name == RunnerName::Host {
            let Some(RunnerCommand::Flash(flash)) = cmd else { unreachable!() };
            const HOST: &str = "target/wasefire/host";
            if fs::exists(HOST).await {
                fs::remove_dir_all(HOST).await?;
            }
            cargo.arg("--");
            cargo.arg(format!("../../{HOST}"));
            let attach = Attach { name: self.name, log: self.log, options: flash.attach };
            attach.execute_host(main, Some(cargo)).await?;
        } else {
            cmd::execute(&mut cargo).await?;
        }
        let elf = self.name.elf(main).await;
        if self.name == RunnerName::Nordic {
            let name = self.name(24)?;
            let version = version.as_deref().unwrap_or("00000000");
            ensure!(version.len() == 8, "--version must be a big-endian hexadecimal u32");
            ensure!(version != "ffffffff", "--version must be smaller than u32::MAX");
            let version = u32::from_be_bytes(HEX.decode(version.as_bytes())?[..].try_into()?);
            let mut content = fs::read(&elf).await?;
            let pos = section_offset(&content, ".header")?;
            content[pos ..][.. 4].copy_from_slice(&version.to_le_bytes());
            content[pos + 0x10 ..][.. 0x18].copy_from_slice(&name);
            fs::write(&elf, content).await?;
        }
        if self.name == RunnerName::OpenTitan {
            let _name = self.name(0)?;
            let mut version = match version.as_deref() {
                Some(x) => HEX.decode(x.as_bytes())?,
                None => vec![0; 20],
            };
            ensure!(version.len() == 20, "--version must be 20 bytes in hexadecimal");
            ensure!(version != [0xff; 20], "--version must not be all 0xff");
            let len = [4, 4, 4, 8].into_iter().fold(0, |pos, len| {
                version[pos ..][.. len].reverse();
                pos + len
            });
            let mut content = fs::read(&elf).await?;
            let pos = section_offset(&content, ".manifest")? + 836;
            content[pos ..][.. len].copy_from_slice(&version);
            fs::write(&elf, content).await?;
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
        let Some(mut cmd) = cmd else { return Ok(()) };
        let flash = match cmd {
            RunnerCommand::Update(cmd_update) => {
                let platform_a = self.bundle(&elf, side).await?.into();
                let transfer = cmd_update.transfer.clone();
                let action = action::PlatformUpdate { platform_a, platform_b: None, transfer };
                action.run(&update.unwrap()).await?;
                if !cmd_update.both || cmd_update.step.is_some() {
                    match cmd_update.command {
                        Some(UpdateCommand::Attach(options)) => {
                            let attach = Attach { name: self.name, log: self.log, options };
                            attach.execute(main).await?
                        }
                        None => return Ok(()),
                    }
                }
                let cmd = RunnerCommand::Update(Update { step: Some(1 - step), ..cmd_update });
                self.version = Some(version.unwrap().into_owned());
                return Box::pin(self.execute(main, Some(cmd))).await;
            }
            RunnerCommand::Flash(x) => x,
            RunnerCommand::Bundle { ref mut step } => {
                self.bundle(&elf, side).await?;
                if *step < max_step {
                    *step += 1;
                    return Box::pin(self.execute(main, Some(cmd))).await;
                }
                return Ok(());
            }
        };
        if self.name == RunnerName::Nordic {
            let board = board.unwrap();
            let board_feature = match board {
                "makerdiary" => "dongle",
                _ => board,
            };
            let mut cargo = Command::new("cargo");
            cargo.current_dir("crates/runner-nordic/crates/bootloader");
            cargo.args(["build", "--release", "--target=thumbv7em-none-eabi"]);
            cargo.arg(format!("--features=board-{board_feature}"));
            if self.single_sided {
                cargo.arg("--features=single-sided");
            }
            cargo.args(["-Zbuild-std=core", "--config=profile.release.panic=\"immediate-abort\""]);
            // TODO(https://github.com/rust-lang/rust/issues/122105): Remove when fixed.
            cargo.env("RUSTFLAGS", "--allow=unused-crate-dependencies");
            cmd::execute(&mut cargo).await?;
            if matches!(board, "dongle" | "makerdiary") {
                let runner = self.bundle(&elf, side).await?;
                let bootloader = "target/thumbv7em-none-eabi/release/bootloader";
                let mut objcopy = wrap_command().await?;
                objcopy.args(["rust-objcopy", bootloader]);
                objcopy.arg(format!("--update-section=.runner={runner}"));
                cmd::execute(&mut objcopy).await?;
                let artifact = "target/wasefire/platform.hex";
                produce_hex(bootloader, artifact).await?;
                if flash.artifacts {
                    instruction(&format!("Flash {artifact}")).await?;
                } else {
                    let mut flash = wrap_command().await?;
                    if board == "dongle" {
                        flash.args(["nrfdfu", bootloader]);
                    } else {
                        flash.args(["uf2conv.py", "--family=0xADA52840", artifact]);
                    }
                    cmd::execute(&mut flash).await?;
                }
                if board != "dongle" {
                    return Ok(());
                }
                // Because the Nordic dongle bootloader checks that the firmware did not change
                // before booting it, we additionally flash the Wasefire bootloader alone so the
                // Nordic bootloader will only check that the Wasefire bootloader did not change.
                // This permits updating the platform.
                instruction("Press the RESET button on the dongle to enter DFU mode").await?;
                let mut objcopy = wrap_command().await?;
                objcopy.args(["rust-objcopy", bootloader]);
                objcopy.arg("--remove-section=.runner");
                objcopy.arg("--remove-section=.header");
                cmd::execute(&mut objcopy).await?;
                if flash.artifacts {
                    let artifact = "target/wasefire/bootloader.hex";
                    produce_hex(bootloader, artifact).await?;
                    instruction(&format!("Flash {artifact}")).await?;
                } else {
                    let mut nrfdfu = wrap_command().await?;
                    nrfdfu.args(["nrfdfu", bootloader]);
                    cmd::execute(&mut nrfdfu).await?;
                }
                return Ok(());
            }
        }
        if self.name == RunnerName::OpenTitan {
            opentitan::execute(main, &flash.attach, &elf).await?;
        }
        if flash.artifacts {
            let artifact = "target/wasefire/platform.hex";
            produce_hex("target/thumbv7em-none-eabi/release/bootloader", artifact).await?;
            let mut bootloader = tokio::fs::read(artifact).await?;
            assert_eq!(bootloader.pop(), Some(b'\n'));
            produce_hex(&elf, artifact).await?;
            let mut platform = tokio::fs::read(artifact).await?;
            assert_eq!(platform.pop(), Some(b'\n'));
            let mut merged = Vec::with_capacity(bootloader.len() + platform.len());
            let mut start: &[u8] = b"";
            for line in bootloader.split(|&x| x == b'\n') {
                match &line[7 .. 9] {
                    b"01" => (),
                    b"03" => start = line,
                    _ => {
                        merged.extend_from_slice(line);
                        merged.push(b'\n');
                    }
                }
            }
            for line in platform.split(|&x| x == b'\n') {
                match &line[7 .. 9] {
                    b"03" => merged.extend_from_slice(start),
                    _ => merged.extend_from_slice(line),
                }
                merged.push(b'\n');
            }
            tokio::fs::write(artifact, &merged).await?;
            instruction(&format!("Erase the device, flash {artifact}")).await?;
            return Ok(());
        }
        let attach = Attach { name: self.name, log: None, options: flash.attach };
        cmd::execute(&mut attach.probe_rs("erase").await?).await?;
        if self.name == RunnerName::Nordic {
            let mut probe_rs = attach.probe_rs("download").await?;
            probe_rs.arg("target/thumbv7em-none-eabi/release/bootloader");
            cmd::execute(&mut probe_rs).await?;
        }
        if self.gdb {
            let chip = self.name.chip();
            println!("Use the following 2 commands in different terminals:");
            println!("JLinkGDBServer -device {chip} -if swd -speed 4000 -port 2331");
            println!("gdb-multiarch -ex 'file {elf}' -ex 'target remote localhost:2331'");
        }
        attach.execute_probe_rs(main, "run").await?
    }

    async fn bundle(&self, elf: &str, side: Option<usize>) -> Result<String> {
        let side = match side {
            None => "",
            Some(0) => "_a",
            Some(1) => "_b",
            _ => unimplemented!(),
        };
        let bundle = format!("target/wasefire/platform{side}.bin");
        match self.name {
            RunnerName::Host => drop(fs::copy(elf, &bundle).await?),
            RunnerName::OpenTitan => {
                let signed = format!("{elf}.{}.signed.bin", opentitan::APPKEY);
                opentitan::build(elf).await?;
                opentitan::truncate(&signed).await?;
                fs::copy(&signed, &bundle).await?;
            }
            _ => {
                let mut objcopy = wrap_command().await?;
                objcopy.args(["rust-objcopy", "--output-target=binary", elf, &bundle]);
                cmd::execute(&mut objcopy).await?;
            }
        }
        Ok(bundle)
    }

    fn name(&self, len: usize) -> Result<Vec<u8>> {
        match (&self.name_hex, &self.name_str) {
            (None, None) => Ok(vec![0; len]),
            (None, Some(name)) => {
                ensure!(name.len() <= len, "--name-str must have at most {len} bytes");
                ensure!(
                    name.bytes().all(|x| x.is_ascii_graphic()),
                    "--name-str must only have graphic ASCII characters"
                );
                let mut result = vec![0; len];
                result[.. name.len()].copy_from_slice(name.as_bytes());
                Ok(result)
            }
            (Some(name), None) => {
                ensure!(name.len() == 2 * len, "--name-hex must have {len} bytes");
                Ok(HEX.decode(name.as_bytes())?)
            }
            (Some(_), Some(_)) => unreachable!(),
        }
    }
}

impl Attach {
    async fn execute(self, main: &MainOptions) -> Result<!> {
        match self.name {
            RunnerName::Host => self.execute_host(main, None).await,
            RunnerName::Nordic => self.execute_probe_rs(main, "attach").await,
            RunnerName::OpenTitan => {
                opentitan::attach(None, main, &self.options, &self.name.elf(main).await).await
            }
        }
    }

    async fn execute_host(self, main: &MainOptions, mut cargo: Option<Command>) -> Result<!> {
        const HOST: &str = "target/wasefire/host";
        let bin = format!("{HOST}/platform.bin");
        loop {
            let copy_bin = cargo.is_some();
            let mut cargo = cargo.take().unwrap_or_else(|| {
                let mut cargo = Command::new(&bin);
                cargo.arg(HOST);
                if let Some(log) = &self.log {
                    cargo.env("RUST_LOG", log);
                }
                cargo
            });
            if std::env::var_os("CODESPACES").is_some() {
                log::warn!("Assuming --protocol=unix when running in a codespace.");
                cargo.arg("--protocol=unix");
            }
            cargo.args(&self.options.args);
            cmd::exit_status(&mut cargo).await?;
            if copy_bin {
                fs::copy(RunnerName::Host.elf(main).await, &bin).await?;
            }
        }
    }

    async fn execute_probe_rs(self, main: &MainOptions, mut cmd: &'static str) -> Result<!> {
        let elf = self.name.elf(main).await;
        loop {
            let mut probe_rs = self.probe_rs(cmd).await?;
            probe_rs.args(["--catch-reset", &elf]);
            cmd::status(&mut probe_rs).await?;
            cmd = "attach";
        }
    }

    async fn probe_rs(&self, cmd: &'static str) -> Result<Command> {
        let chip = self.name.chip();
        let mut probe_rs = wrap_command().await?;
        probe_rs.args(["probe-rs", cmd]);
        probe_rs.arg(format!("--chip={chip}"));
        probe_rs.args(&self.options.args);
        Ok(probe_rs)
    }
}

impl Wait {
    async fn execute(&self, applet: bool) -> Result<()> {
        let period = Duration::from_millis(300);
        loop {
            tokio::time::sleep(period).await;
            let mut action = action::AppletExitStatus::parse_from::<_, OsString>([]);
            action.wait.set_period(period);
            if applet {
                action.ensure_exit();
            }
            let connection = match self.options.connect().await {
                Ok(x) => x,
                Err(_) => continue,
            };
            let error = match action.run(&connection).await {
                Err(x) => x,
                Ok(()) => continue,
            };
            use wasefire_error::{Code, Error};
            if root_cause_is::<Error>(&error, |&x| x == Error::user(Code::NotFound)) {
                break Ok(());
            }
        }
    }
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
    const ASC_VERSION: &str = "0.28.9"; // scripts/upgrade.sh relies on this name
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

fn section_offset(elf: &[u8], name: &str) -> Result<usize> {
    let file = <ElfFile32>::parse(elf)?;
    let (_, header) = file
        .elf_section_table()
        .section_by_name(file.endian(), name.as_bytes())
        .with_context(|| anyhow!("no {name} section"))?;
    Ok(header.sh_offset(file.endian()) as usize)
}

async fn produce_hex(elf: &str, hex: &str) -> Result<()> {
    let mut objcopy = wrap_command().await?;
    objcopy.args(["rust-objcopy", "--output-target=ihex", elf, hex]);
    cmd::execute(&mut objcopy).await
}

async fn instruction(msg: &str) -> Result<()> {
    println!("\x1b[1;33mTODO\x1b[m: {msg}, then hit ENTER.");
    std::io::stdin().read_line(&mut String::new())?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("warn"));
    Flags::parse().execute().await?;
    Ok(())
}
