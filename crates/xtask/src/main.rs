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

use std::cell::Cell;
use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::fmt::Display;
use std::num::ParseIntError;
use std::os::unix::prelude::CommandExt;
use std::process::{Command, Output};
use std::str::FromStr;

use anyhow::{bail, ensure, Context, Result};
use cargo_metadata::MetadataCommand;
use clap::Parser;
use lazy_static::lazy_static;
use probe_rs::config::TargetSelector;
use probe_rs::{flashing, Permissions, Session};
use rustc_demangle::demangle;
use sha2::{Digest, Sha256};
use strum::{Display, EnumString};

mod footprint;
mod fs;
mod lazy;

#[derive(Parser)]
struct Flags {
    #[clap(flatten)]
    options: MainOptions,

    #[clap(subcommand)]
    command: MainCommand,
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

    /// Updates the applet API for all languages.
    UpdateApis {
        /// Cargo features.
        #[clap(long, default_values_t = ["full-api".to_string()])]
        features: Vec<String>,
    },

    /// Appends a comparison between footprint-base.toml and footprint-pull_request.toml.
    ///
    /// If any file is missing, it is assumed to have no measurements.
    Footprint {
        /// The markdown table is written to this file.
        output: String,
    },
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
    #[clap(long, default_value = "release")]
    profile: String,

    /// Cargo features.
    #[clap(long)]
    features: Vec<String>,

    /// Optimization level (0, 1, 2, 3, s, z).
    #[clap(long, short = 'O')]
    opt_level: Option<OptLevel>,

    /// Stack size.
    #[clap(long, default_value_t)]
    stack_size: StackSize,

    /// Whether to call wasm-strip on the applet.
    #[clap(skip = Cell::new(true))]
    strip: Cell<bool>,

    /// Whether to call wasm-opt on the applet.
    #[clap(skip = Cell::new(true))]
    opt: Cell<bool>,
}

#[derive(clap::Subcommand)]
enum AppletCommand {
    /// Compiles a runner with the applet.
    Runner(RunnerOptions),

    /// Runs twiggy on the applet.
    ///
    /// If an argument is "APPLET", then it is replaced with the applet path. At most one argument
    /// may be "APPLET". If none are used, the applet path is appended at the end.
    ///
    /// A typical example would be `cargo xtask applet lang name twiggy -- top`.
    Twiggy {
        #[clap(last = true)]
        args: Vec<String>,
    },
}

#[derive(clap::Args)]
struct Runner {
    #[clap(flatten)]
    options: RunnerOptions,
}

#[derive(Default, clap::Args)]
struct RunnerOptions {
    /// Runner name.
    name: String,

    /// Platform version.
    ///
    /// How the version string is interpreted is up to the runner. For Nordic, it must be a u32
    /// smaller than u32::MAX.
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
    opt_level: Option<OptLevel>,

    /// Produces target/wasefire/platform_{side}.bin files instead of flashing.
    #[clap(long)]
    bundle: bool,

    /// Resets the persistent storage before running.
    #[clap(long)]
    reset_storage: bool,

    /// Prints the command lines to use GDB.
    #[clap(long)]
    gdb: bool,

    /// Defmt log filter.
    #[clap(long)]
    log: Option<String>,

    /// Creates a web interface for the host runner.
    #[clap(long)]
    web: bool,

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

#[derive(Copy, Clone)]
struct StackSize(usize);

impl Default for StackSize {
    fn default() -> Self {
        Self(16384)
    }
}

impl Display for StackSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for StackSize {
    type Err = ParseIntError;

    fn from_str(input: &str) -> std::result::Result<Self, Self::Err> {
        Ok(StackSize(usize::from_str(input)?))
    }
}

#[derive(Copy, Clone, EnumString, Display)]
enum OptLevel {
    #[strum(serialize = "0")]
    O0,
    #[strum(serialize = "1")]
    O1,
    #[strum(serialize = "2")]
    O2,
    #[strum(serialize = "3")]
    O3,
    #[strum(serialize = "s")]
    Os,
    #[strum(serialize = "z")]
    Oz,
}

impl Flags {
    fn execute(self) -> Result<()> {
        match self.command {
            MainCommand::Applet(applet) => applet.execute(&self.options)?,
            MainCommand::Runner(runner) => runner.execute(&self.options)?,
            MainCommand::UpdateApis { features } => {
                let (lang, ext) = ("assemblyscript", "ts");
                let mut cargo = Command::new("cargo");
                cargo.args(["run", "--manifest-path=crates/api-desc/Cargo.toml"]);
                for features in features {
                    cargo.arg(format!("--features={features}"));
                }
                cargo.arg("--");
                cargo.arg(format!("--lang={lang}"));
                cargo.arg(format!("--output=examples/{lang}/api.{ext}"));
                execute_command(&mut cargo)?;
            }
            MainCommand::Footprint { output } => footprint::compare(&output)?,
        }
        Ok(())
    }
}

impl Applet {
    fn execute(&self, main: &MainOptions) -> Result<()> {
        if matches!(self.command, Some(AppletCommand::Twiggy { .. })) {
            self.options.strip.set(false);
            // TODO(https://github.com/rustwasm/twiggy/issues/326): Twiggy returns "should not parse
            // the same key into multiple items" when using wasm-opt. Ideally we would be able to
            // use twiggy on the wasm-opt output.
            self.options.opt.set(false);
        }
        self.options.execute(main, &self.command)?;
        if let Some(command) = &self.command {
            command.execute(main)?;
        }
        Ok(())
    }
}

impl AppletOptions {
    fn execute(&self, main: &MainOptions, command: &Option<AppletCommand>) -> Result<()> {
        match self.lang.as_str() {
            "rust" => self.execute_rust(main, command),
            "assemblyscript" => self.execute_assemblyscript(main),
            x => bail!("unsupported language {x}"),
        }
    }

    fn execute_rust(&self, main: &MainOptions, command: &Option<AppletCommand>) -> Result<()> {
        let dir = if self.name.starts_with(['.', '/']) {
            self.name.clone()
        } else {
            format!("examples/{}/{}", self.lang, self.name)
        };
        ensure!(fs::exists(&dir), "{dir} does not exist");
        let native = match (main.native, &main.native_target, command) {
            (_, Some(target), command) => {
                if let Some(AppletCommand::Runner(x)) = command {
                    ensure!(target == x.target(), "--native-target must match runner");
                }
                Some(target.as_str())
            }
            (true, None, Some(AppletCommand::Runner(x))) => Some(x.target()),
            (true, None, _) => bail!("--native requires runner"),
            (false, _, _) => None,
        };
        let metadata = MetadataCommand::new().current_dir(&dir).no_deps().exec()?;
        let target_dir = &metadata.target_directory;
        assert_eq!(metadata.packages.len(), 1);
        let name = metadata.packages[0].name.replace('-', "_");
        let out = match native {
            None => format!("{target_dir}/wasm32-unknown-unknown/release/{name}.wasm"),
            Some(target) => format!("{target_dir}/{target}/release/lib{name}.a"),
        };
        let mut cargo = Command::new("cargo");
        let mut rustflags = vec![
            "-C panic=abort".to_string(),
            "-C codegen-units=1".to_string(),
            "-C embed-bitcode=yes".to_string(),
            "-C lto=fat".to_string(),
        ];
        cargo.args(["rustc", "--lib"]);
        match native {
            None => {
                rustflags.push(format!("-C link-arg=-zstack-size={}", self.stack_size));
                cargo.args(["--crate-type=cdylib", "--target=wasm32-unknown-unknown"]);
            }
            Some(target) => {
                cargo.args(["--crate-type=staticlib", "--features=wasefire/native"]);
                cargo.arg(format!("--target={target}"));
            }
        }
        cargo.arg(format!("--profile={}", self.profile));
        if let Some(level) = self.opt_level {
            rustflags.push(format!("-C opt-level={level}"));
        }
        for features in &self.features {
            cargo.arg(format!("--features={features}"));
        }
        if main.release {
            cargo.args(["-Zbuild-std=core,alloc", "-Zbuild-std-features=panic_immediate_abort"]);
        } else {
            cargo.env("WASEFIRE_DEBUG", "");
        }
        cargo.env("RUSTFLAGS", rustflags.join(" "));
        cargo.current_dir(dir);
        execute_command(&mut cargo)?;
        let applet = match native {
            Some(_) => "target/wasefire/libapplet.a",
            None => "target/wasefire/applet.wasm",
        };
        let changed = copy_if_changed(&out, applet)?;
        if native.is_some() {
            if main.size {
                let mut size = wrap_command()?;
                size.args(["rust-size", applet]);
                let output = String::from_utf8(output_command(&mut size)?.stdout)?;
                // We assume the interesting part is the first line after the header.
                for line in output.lines().take(2) {
                    println!("{line}");
                }
            }
            if let Some(key) = &main.footprint {
                footprint::update_applet(key, footprint::rust_size(applet)?)?;
            }
        }
        if native.is_none() && changed {
            self.optimize_wasm(main)?;
        }
        Ok(())
    }

    fn execute_assemblyscript(&self, main: &MainOptions) -> Result<()> {
        let dir = format!("examples/{}", self.lang);
        ensure_assemblyscript()?;
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
        execute_command(&mut asc)?;
        self.optimize_wasm(main)
    }

    fn optimize_wasm(&self, main: &MainOptions) -> Result<()> {
        let wasm = "target/wasefire/applet.wasm";
        if main.size {
            println!("Applet size: {}", fs::metadata(wasm)?.len());
        }
        if self.strip.get() {
            let mut strip = wrap_command()?;
            strip.arg("wasm-strip");
            strip.arg(wasm);
            execute_command(&mut strip)?;
            if main.size {
                println!("Applet size (after wasm-strip): {}", fs::metadata(wasm)?.len());
            }
        }
        if self.opt.get() {
            let mut opt = wrap_command()?;
            opt.arg("wasm-opt");
            opt.args(["--enable-bulk-memory", "--enable-sign-ext"]);
            match self.opt_level {
                Some(level) => drop(opt.arg(format!("-O{level}"))),
                None => drop(opt.arg("-O")),
            }
            opt.args([wasm, "-o", wasm]);
            execute_command(&mut opt)?;
            if main.size {
                println!("Applet size (after wasm-opt): {}", fs::metadata(wasm)?.len());
            }
        }
        if let Some(key) = &main.footprint {
            footprint::update_applet(key, fs::metadata(wasm)?.len() as usize)?;
        }
        Ok(())
    }
}

impl AppletCommand {
    fn execute(&self, main: &MainOptions) -> Result<()> {
        match self {
            AppletCommand::Runner(runner) => runner.execute(main, 0, true),
            AppletCommand::Twiggy { args } => {
                let mut twiggy = wrap_command()?;
                twiggy.arg("twiggy");
                let mut wasm = Some("target/wasefire/applet.wasm");
                for arg in args {
                    let _ = match arg.as_str() {
                        "APPLET" => twiggy.arg(wasm.take().unwrap()),
                        _ => twiggy.arg(arg),
                    };
                }
                wasm.map(|x| twiggy.arg(x));
                execute_command(&mut twiggy)
            }
        }
    }
}

impl Runner {
    fn execute(&self, main: &MainOptions) -> Result<()> {
        self.options.execute(main, 0, false)?;
        Ok(())
    }
}

impl RunnerOptions {
    fn execute(&self, main: &MainOptions, step: usize, run: bool) -> Result<()> {
        let mut cargo = Command::new("cargo");
        let mut rustflags = Vec::new();
        let mut features = self.features.clone();
        if run && self.name == "host" {
            cargo.arg("run");
        } else {
            cargo.arg("build");
        }
        cargo.arg("--release");
        cargo.arg(format!("--target={}", self.target()));
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
                cargo.arg("-Zbuild-std-features=panic_immediate_abort");
            }
            if main.release {
                rustflags.push("-C lto=fat".to_string());
                rustflags.push("-C codegen-units=1".to_string());
                rustflags.push("-C embed-bitcode=yes".to_string());
            } else {
                rustflags.push("-C link-arg=-Tdefmt.x".to_string());
                rustflags.push("-C debuginfo=2".to_string());
            }
        }
        if let Some(level) = self.opt_level {
            rustflags.push(format!("-C opt-level={level}"));
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
        if self.name == "host" && self.web {
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
        fs::touch("target/wasefire/applet.wasm")?;
        if run && self.name == "host" {
            let path = "target/wasefire/storage.bin";
            if self.reset_storage && fs::exists(path) {
                fs::remove_file(path)?;
            }
            replace_command(cargo);
        } else {
            execute_command(&mut cargo)?;
        }
        if self.measure_bloat {
            ensure_command(&["cargo", "bloat"])?;
            let mut bloat = wrap_command()?;
            bloat.arg(cargo.get_program());
            if let Some(dir) = cargo.get_current_dir() {
                bloat.current_dir(dir);
            }
            for (key, val) in cargo.get_envs() {
                match val {
                    None => bloat.env_remove(key),
                    Some(val) => bloat.env(key, val),
                };
            }
            for arg in cargo.get_args() {
                if arg == "build" {
                    bloat.arg("bloat");
                } else {
                    bloat.arg(arg);
                }
            }
            bloat.args(["--crates", "--split-std"]);
            execute_command(&mut bloat)?;
        }
        let elf = self.board_target();
        if main.size {
            let mut size = wrap_command()?;
            size.arg("rust-size");
            size.arg(&elf);
            execute_command(&mut size)?;
        }
        if let Some(key) = &main.footprint {
            footprint::update_runner(key, footprint::rust_size(&elf)?)?;
        }
        if let Some(stack_sizes) = self.stack_sizes {
            let elf = fs::read(&elf)?;
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
        if self.bundle {
            let mut objcopy = wrap_command()?;
            objcopy.args(["rust-objcopy", "-O", "binary", &elf]);
            objcopy.arg(format!("target/wasefire/platform{side}.bin"));
            execute_command(&mut objcopy)?;
            if step < max_step {
                return self.execute(main, step + 1, run);
            }
            return Ok(());
        }
        if !run {
            return Ok(());
        }
        let chip = match self.name.as_str() {
            "nordic" => "nRF52840_xxAA",
            "host" => unreachable!(),
            _ => unimplemented!(),
        };
        let mut session = lazy::Lazy::new(|| {
            Ok(Session::auto_attach(
                TargetSelector::Unspecified(chip.to_string()),
                Permissions::default(),
            )?)
        });
        if self.reset_storage {
            println!("Erasing the persistent storage.");
            // Keep those values in sync with crates/runner-nordic/memory.x.
            flashing::erase_sectors(session.get()?, None, 240, 16)?;
        }
        if self.name == "nordic" {
            let mut cargo = Command::new("cargo");
            cargo.current_dir("crates/runner-nordic/crates/bootloader");
            cargo.args(["build", "--release", "--target=thumbv7em-none-eabi"]);
            cargo.args(["-Zbuild-std=core", "-Zbuild-std-features=panic_immediate_abort"]);
            execute_command(&mut cargo)?;
            flashing::download_file(
                session.get()?,
                "target/thumbv7em-none-eabi/release/bootloader",
                flashing::Format::Elf,
            )?;
        }
        if self.gdb {
            println!("Use the following 2 commands in different terminals:");
            println!("JLinkGDBServer -device {chip} -if swd -speed 4000 -port 2331");
            println!("gdb-multiarch -ex 'file {elf}' -ex 'target remote localhost:2331'");
        }
        let mut probe_rs = wrap_command()?;
        probe_rs.args(["probe-rs", "run"]);
        probe_rs.arg(format!("--chip={chip}"));
        probe_rs.arg(elf);
        println!("Replace `run` with `attach` in the following command to rerun:");
        replace_command(probe_rs);
    }

    fn target(&self) -> &'static str {
        lazy_static! {
            // Each time we specify RUSTFLAGS, we want to specify --target. This is because if
            // --target is not specified then RUSTFLAGS applies to all compiler invocations
            // (including build scripts and proc macros). This leads to recompilation when RUSTFLAGS
            // changes. See https://github.com/rust-lang/cargo/issues/8716.
            static ref HOST_TARGET: String = {
                let mut sh = Command::new("sh");
                sh.args(["-c", "rustc -vV | sed -n 's/^host: //p'"]);
                read_output_line(&mut sh).unwrap()
            };
        }
        match self.name.as_str() {
            "nordic" => "thumbv7em-none-eabi",
            "host" => &HOST_TARGET,
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

    fn board_target(&self) -> String {
        format!("target/{}/release/runner-{}", self.target(), self.name)
    }
}

fn execute_command(command: &mut Command) -> Result<()> {
    println!("{command:?}");
    let code = command.spawn()?.wait()?.code().context("no error code")?;
    ensure!(code == 0, "failed with code {code}");
    Ok(())
}

fn replace_command(mut command: Command) -> ! {
    println!("{command:?}");
    panic!("{}", command.exec());
}

fn output_command(command: &mut Command) -> Result<Output> {
    println!("{command:?}");
    let output = command.output()?;
    ensure!(output.status.success(), "failed with status {}", output.status);
    Ok(output)
}

fn read_output_line(command: &mut Command) -> Result<String> {
    let mut output = output_command(command)?;
    assert!(output.stderr.is_empty());
    assert_eq!(output.stdout.pop(), Some(b'\n'));
    Ok(String::from_utf8(output.stdout)?)
}

fn ensure_command(cmd: &[&str]) -> Result<()> {
    let mut wrapper = Command::new("./scripts/wrapper.sh");
    wrapper.args(cmd);
    wrapper.env("WASEFIRE_WRAPPER_EXEC", "n");
    execute_command(&mut wrapper)
}

fn wrap_command() -> Result<Command> {
    Ok(Command::new(fs::canonicalize("./scripts/wrapper.sh")?))
}

/// Copies a file if its destination .hash changed.
///
/// Returns whether the copy took place.
fn copy_if_changed(src: &str, dst: &str) -> Result<bool> {
    let dst_hash = format!("{dst}.hash");
    let src_hash = Sha256::digest(fs::read(src)?);
    let changed = !fs::exists(dst) || !fs::exists(&dst_hash) || fs::read(&dst_hash)? != *src_hash;
    if changed {
        println!("cp {src} {dst}");
        fs::copy(src, dst)?;
        fs::write(&dst_hash, src_hash)?;
    }
    Ok(changed)
}

fn ensure_assemblyscript() -> Result<()> {
    const ASC_VERSION: &str = "0.27.22"; // scripts/upgrade.sh relies on this name
    const BIN: &str = "examples/assemblyscript/node_modules/.bin/asc";
    const JSON: &str = "examples/assemblyscript/node_modules/assemblyscript/package.json";
    if fs::exists(BIN) && fs::exists(JSON) {
        let mut sed = Command::new("sed");
        sed.args(["-n", r#"s/^  "version": "\(.*\)",$/\1/p"#, JSON]);
        if read_output_line(&mut sed)? == ASC_VERSION {
            return Ok(());
        }
    }
    ensure_command(&["npm"])?;
    let mut npm = wrap_command()?;
    npm.args(["npm", "install", "--no-save"]);
    npm.arg(format!("assemblyscript@{ASC_VERSION}"));
    npm.current_dir("examples/assemblyscript");
    execute_command(&mut npm)
}

fn main() -> Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("warn"));
    Flags::parse().execute()?;
    Ok(())
}
