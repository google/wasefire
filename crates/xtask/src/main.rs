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

use std::cell::Cell;
use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::fmt::Display;
use std::num::ParseIntError;
use std::os::unix::prelude::CommandExt;
use std::path::Path;
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

mod fs;

#[derive(Parser)]
struct Flags {
    #[clap(flatten)]
    options: MainOptions,

    #[clap(subcommand)]
    command: MainCommand,
}

#[derive(clap::Args)]
struct MainOptions {
    /// (unstable) Compiles with multivalue support.
    #[clap(long)]
    multivalue: bool,

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
    // TODO: Add a flag to add "-C link-arg=-Map=output.map" to get the map of why the linker
    // added/kept something.
}

#[derive(clap::Subcommand)]
enum MainCommand {
    /// Compiles an applet.
    Applet(Applet),

    /// Compiles a runner.
    Runner(Runner),

    /// Updates the applet API for all languages.
    UpdateApis,
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
    #[clap(long, short = 'O', default_value_t)]
    opt_level: OptLevel,

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

    /// Cargo no-default-features.
    #[clap(long)]
    no_default_features: bool,

    /// Cargo features.
    #[clap(long)]
    features: Vec<String>,

    /// Optimization level (0, 1, 2, 3, s, z).
    #[clap(long, short = 'O', default_value_t)]
    opt_level: OptLevel,

    /// Erases all the flash first.
    #[clap(long)]
    erase_flash: bool,

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

    /// Show the (top N) stack sizes of the firmware
    #[clap(long)]
    stack_sizes: Option<Option<usize>>,
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

#[derive(Default, Copy, Clone, EnumString, Display)]
enum OptLevel {
    #[strum(serialize = "0")]
    O0,
    #[strum(serialize = "1")]
    O1,
    #[strum(serialize = "2")]
    O2,
    #[strum(serialize = "3")]
    #[default]
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
            MainCommand::UpdateApis => {
                let (lang, ext) = ("assemblyscript", "ts");
                let mut cargo = Command::new("cargo");
                cargo.args(["run", "--manifest-path=crates/api-desc/Cargo.toml", "--"]);
                cargo.arg(format!("--lang={lang}"));
                cargo.arg(format!("--output=examples/{lang}/api.{ext}"));
                execute_command(&mut cargo)?;
            }
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
        ensure!(Path::new(&dir).exists(), "{dir} does not exist");
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
            format!("-C opt-level={}", self.opt_level),
            "-C lto=fat".to_string(),
        ];
        cargo.args(["rustc", "--lib"]);
        match native {
            None => {
                rustflags.push(format!("-C link-arg=-zstack-size={}", self.stack_size));
                if main.multivalue {
                    rustflags.push("-C target-feature=+multivalue".to_string());
                }
                cargo.args(["--crate-type=cdylib", "--target=wasm32-unknown-unknown"]);
            }
            Some(target) => {
                cargo.args(["--crate-type=staticlib", "--features=wasefire/native"]);
                cargo.arg(format!("--target={target}"));
            }
        }
        cargo.arg(format!("--profile={}", self.profile));
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
        if native.is_some() {
            copy_if_changed(&out, "target/wasefire/libapplet.a")?;
        } else if copy_if_changed(&out, "target/wasefire/applet.wasm")? {
            self.optimize_wasm(main)?;
        }
        Ok(())
    }

    fn execute_assemblyscript(&self, main: &MainOptions) -> Result<()> {
        let dir = format!("examples/{}", self.lang);
        ensure_assemblyscript()?;
        let mut asc = Command::new("./node_modules/.bin/asc");
        asc.args(["-o", "../../target/wasefire/applet.wasm"]);
        asc.arg(format!("-O{}", self.opt_level));
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
            println!("Initial applet size: {}", fs::metadata(wasm)?.len());
        }
        if self.strip.get() {
            let mut strip = wrap_command()?;
            strip.arg("wasm-strip");
            strip.arg(wasm);
            execute_command(&mut strip)?;
            if main.size {
                println!("Stripped applet size: {}", fs::metadata(wasm)?.len());
            }
        }
        if self.opt.get() {
            let mut opt = wrap_command()?;
            opt.arg("wasm-opt");
            if main.multivalue {
                opt.arg("--enable-multivalue");
            }
            opt.args([
                "--enable-bulk-memory",
                "--enable-sign-ext",
                &format!("-O{}", self.opt_level),
            ]);
            opt.args([wasm, "-o", wasm]);
            execute_command(&mut opt)?;
            if main.size {
                println!("Optimized applet size: {}", fs::metadata(wasm)?.len());
            }
        }
        Ok(())
    }
}

impl AppletCommand {
    fn execute(&self, main: &MainOptions) -> Result<()> {
        match self {
            AppletCommand::Runner(runner) => runner.execute(main, true),
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
        self.options.execute(main, false)?;
        Ok(())
    }
}

impl RunnerOptions {
    fn execute(&self, main: &MainOptions, run: bool) -> Result<()> {
        let mut cargo = Command::new("cargo");
        let mut rustflags = Vec::new();
        if run && self.name == "host" {
            cargo.arg("run");
        } else {
            cargo.arg("build");
        }
        cargo.arg("--release");
        cargo.arg(format!("--target={}", self.target()));
        if self.name == "nordic" {
            rustflags.extend([
                "-C link-arg=--nmagic".to_string(),
                "-C link-arg=-Tlink.x".to_string(),
                "-C codegen-units=1".to_string(),
                "-C embed-bitcode=yes".to_string(),
            ]);
            if main.release {
                // We have to split -Z from its argument because of cargo bloat.
                cargo.args([
                    "-Z",
                    "build-std=core,alloc",
                    "-Z",
                    "build-std-features=panic_immediate_abort",
                ]);
            }
            if main.release {
                rustflags.push("-C lto=fat".to_string());
            } else {
                rustflags.push("-C link-arg=-Tdefmt.x".to_string());
                rustflags.push("-C debuginfo=2".to_string());
            }
        }
        rustflags.push(format!("-C opt-level={}", self.opt_level));
        if main.release {
            cargo.arg("--features=release");
        } else {
            cargo.arg("--features=debug");
        }
        if self.no_default_features {
            cargo.arg("--no-default-features");
        } else if std::env::var_os("CODESPACES").is_some() {
            log::warn!("Assuming runner --no-default-features when running in a codespace.");
            cargo.arg("--no-default-features");
        }
        for features in &self.features {
            cargo.arg(format!("--features={features}"));
        }
        if let Some(log) = &self.log {
            cargo.env(self.log_env(), log);
        }
        if self.name == "host" && self.web {
            cargo.arg("--features=web");
        }
        if self.stack_sizes.is_some() {
            rustflags.push("-Z emit-stack-sizes".to_string());
            rustflags.push("-C link-arg=-Tstack-sizes.x".to_string());
        }
        if main.native {
            cargo.arg("--features=native");
        } else {
            cargo.arg("--features=wasm");
        }
        cargo.env("RUSTFLAGS", rustflags.join(" "));
        cargo.current_dir(format!("crates/runner-{}", self.name));
        fs::touch("target/wasefire/applet.wasm")?;
        if run && self.name == "host" {
            let path = Path::new("target/wasefire/storage.bin");
            if self.erase_flash && path.exists() {
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
        if !run {
            return Ok(());
        }
        let chip = match self.name.as_str() {
            "nordic" => "nRF52840_xxAA",
            "host" => unreachable!(),
            _ => unimplemented!(),
        };
        if self.erase_flash {
            let mut session = Session::auto_attach(
                TargetSelector::Unspecified(chip.to_string()),
                Permissions::default(),
            )?;
            println!("Erasing the flash of {}", session.target().name);
            flashing::erase_all(&mut session, None)?;
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
        println!("Add --no-flash to the following command to rerun:");
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
    let dst_file = format!("{dst}.hash");
    let src_hash = Sha256::digest(fs::read(src)?);
    let dst_path = Path::new(dst);
    let changed =
        !dst_path.exists() || !Path::new(&dst_file).exists() || fs::read(&dst_file)? != *src_hash;
    if changed {
        println!("cp {src} {dst}");
        fs::copy(src, dst)?;
        fs::write(&dst_file, src_hash)?;
    }
    Ok(changed)
}

fn ensure_assemblyscript() -> Result<()> {
    const ASC_VERSION: &str = "0.27.17"; // scripts/upgrade.sh relies on this name
    const PATH: &str = "examples/assemblyscript/node_modules/assemblyscript/package.json";
    if Path::new(PATH).exists() {
        let mut sed = Command::new("sed");
        sed.args(["-n", r#"s/^  "version": "\(.*\)",$/\1/p"#, PATH]);
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
