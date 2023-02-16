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

use std::cmp::Reverse;
use std::collections::{BTreeMap, BTreeSet, BinaryHeap};
use std::fmt::Display;
use std::num::ParseIntError;
use std::os::unix::prelude::CommandExt;
use std::path::Path;
use std::process::Command;
use std::str::FromStr;

use anyhow::Result;
use clap::Parser;
use probe_rs::config::TargetSelector;
use probe_rs::{flashing, Permissions, Session};
use rustc_demangle::demangle;
use strum::{Display, EnumString};

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

    /// Lists the available applets.
    ListApplets,

    /// Compiles all applets.
    BuildApplets,

    /// Compiles all runners.
    BuildRunners,

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

    /// Applet name.
    name: String,

    /// Optimization level (0, 1, 2, 3, s, z).
    #[clap(long, short = 'O', default_value_t)]
    opt_level: OptLevel,

    /// Stack size.
    #[clap(long, default_value_t)]
    stack_size: StackSize,
}

#[derive(clap::Subcommand)]
enum AppletCommand {
    /// Compiles a runner with the applet.
    Runner(RunnerOptions),

    /// Runs twiggy on the applet.
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

    /// Enables probe-run --measure-stack flag.
    #[clap(long)]
    measure_stack: bool,

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
            MainCommand::ListApplets => {
                for (lang, applets) in get_applets()? {
                    println!("{lang}:");
                    for applet in applets {
                        println!("- {applet}");
                    }
                }
            }
            MainCommand::BuildApplets => {
                for (lang, applets) in get_applets()? {
                    for name in applets {
                        let applet = Applet {
                            options: AppletOptions {
                                lang: lang.clone(),
                                name,
                                ..AppletOptions::default()
                            },
                            command: None,
                        };
                        applet.execute(&self.options)?;
                    }
                }
            }
            MainCommand::BuildRunners => {
                for runner in std::fs::read_dir("crates")? {
                    let name = runner?.file_name().to_string_lossy().into_owned();
                    if let Some(name) = name.strip_prefix("runner-") {
                        let runner = Runner {
                            options: RunnerOptions {
                                name: name.to_string(),
                                ..RunnerOptions::default()
                            },
                        };
                        runner.execute(&self.options)?;
                    }
                }
            }
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
        self.options.execute(main)?;
        if let Some(command) = &self.command {
            command.execute(main)?;
        }
        Ok(())
    }
}

impl AppletOptions {
    fn execute(&self, main: &MainOptions) -> Result<()> {
        match self.lang.as_str() {
            "rust" => self.execute_rust(main),
            "assemblyscript" => self.execute_assemblyscript(main),
            _ => panic!("unsupported language"),
        }
    }

    fn execute_rust(&self, main: &MainOptions) -> Result<()> {
        let dir = format!("examples/{}/{}", self.lang, self.name);
        let mut cargo = Command::new("cargo");
        let mut rustflags = vec![
            format!("-C link-arg=-zstack-size={}", self.stack_size),
            "-C panic=abort".to_string(),
            "-C codegen-units=1".to_string(),
            "-C embed-bitcode=yes".to_string(),
            format!("-C opt-level={}", self.opt_level),
            "-C lto=fat".to_string(),
        ];
        if main.multivalue {
            rustflags.push("-C target-feature=+multivalue".to_string());
        }
        cargo.args(["build", "--target=wasm32-unknown-unknown", "--release"]);
        if main.release {
            cargo.args(["-Zbuild-std=core,alloc", "-Zbuild-std-features=panic_immediate_abort"]);
        } else {
            cargo.env("FIRWASM_DEBUG", "");
        }
        cargo.env("RUSTFLAGS", rustflags.join(" "));
        cargo.current_dir(dir);
        execute_command(&mut cargo)?;
        std::fs::copy(wasm_target(&self.name), "target/applet.wasm")?;
        self.execute_wasm(main)
    }

    fn execute_assemblyscript(&self, main: &MainOptions) -> Result<()> {
        let dir = format!("examples/{}", self.lang);
        let mut asc = Command::new("./node_modules/.bin/asc");
        asc.args(["-o", "../../target/applet.wasm"]);
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
        self.execute_wasm(main)
    }

    fn execute_wasm(&self, main: &MainOptions) -> Result<()> {
        let wasm = "target/applet.wasm";
        if main.size {
            println!("Initial applet size: {}", std::fs::metadata(wasm)?.len());
        }
        let mut strip = Command::new("wasm-strip");
        strip.arg(wasm);
        execute_command(&mut strip)?;
        if main.size {
            println!("Stripped applet size: {}", std::fs::metadata(wasm)?.len());
        }
        let mut opt = Command::new("wasm-opt");
        if main.multivalue {
            opt.arg("--enable-multivalue");
        }
        opt.args(["--enable-bulk-memory", &format!("-O{}", self.opt_level)]);
        opt.args([wasm, "-o", wasm]);
        execute_command(&mut opt)?;
        if main.size {
            println!("Optimized applet size: {}", std::fs::metadata(wasm)?.len());
        }
        Ok(())
    }
}

impl AppletCommand {
    fn execute(&self, main: &MainOptions) -> Result<()> {
        match self {
            AppletCommand::Runner(runner) => runner.execute(main, true),
            AppletCommand::Twiggy { args } => {
                let mut twiggy = Command::new("twiggy");
                let mut wasm = Some("target/applet.wasm");
                for arg in args {
                    if arg == "APPLET" {
                        twiggy.arg(wasm.take().unwrap());
                    } else {
                        twiggy.arg(arg);
                    }
                }
                if let Some(wasm) = wasm {
                    twiggy.arg(wasm);
                }
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
        if let Some(target) = self.target() {
            cargo.arg(format!("--target={target}"));
        }
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
        if let Some(log) = &self.log {
            cargo.env(self.log_env(), log);
        }
        if self.stack_sizes.is_some() {
            rustflags.push("-Z emit-stack-sizes".to_string());
            rustflags.push("-C link-arg=-Tstack-sizes.x".to_string());
        }
        cargo.env("RUSTFLAGS", rustflags.join(" "));
        cargo.current_dir(format!("crates/runner-{}", self.name));
        if run && self.name == "host" {
            let path = Path::new("target/storage.bin");
            if self.erase_flash && path.exists() {
                std::fs::remove_file(path)?;
            }
            replace_command(cargo);
        } else {
            execute_command(&mut cargo)?;
        }
        if self.measure_bloat {
            let mut bloat = Command::new(cargo.get_program());
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
            let mut size = Command::new("rust-size");
            size.arg(&elf);
            execute_command(&mut size)?;
        }
        if let Some(stack_sizes) = self.stack_sizes {
            let elf = std::fs::read(&elf)?;
            let symbols = stack_sizes::analyze_executable(&elf).unwrap();
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
                let name = *symbol.names().first().expect("missing symbol");
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
            eprintln!("Erasing the flash of {}", session.target().name);
            flashing::erase_all(&mut session, None)?;
        }
        if self.gdb {
            println!("Use the following 2 commands in different terminals:");
            println!("JLinkGDBServer -device {chip} -if swd -speed 4000 -port 2331");
            println!("gdb-multiarch -ex 'file {elf}' -ex 'target remote localhost:2331'");
        }
        let mut probe_run = Command::new("probe-run");
        probe_run.arg(format!("--chip={chip}"));
        if main.release {
            probe_run.arg("--backtrace=never");
        }
        if self.measure_stack {
            probe_run.arg("--measure-stack");
        }
        probe_run.arg(elf);
        replace_command(probe_run);
    }

    fn target(&self) -> Option<&'static str> {
        match self.name.as_str() {
            "nordic" => Some("thumbv7em-none-eabi"),
            "host" => None,
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
        let mut target = "target/".to_string();
        if let Some(name) = self.target() {
            target.push_str(name);
            target.push('/');
        }
        target.push_str("release/runner-");
        target.push_str(&self.name);
        target
    }
}

fn wasm_target(name: &str) -> String {
    format!("target/wasm32-unknown-unknown/release/{name}.wasm")
}

fn get_applets() -> Result<BTreeMap<String, BTreeSet<String>>> {
    let mut result = BTreeMap::new();
    for lang in std::fs::read_dir("examples")? {
        let lang = lang?;
        let lang_name = lang.file_name().to_string_lossy().into_owned();
        if !lang.file_type()?.is_dir() {
            log::warn!("Non-directory {lang_name} in examples.");
            continue;
        }
        let result: &mut BTreeSet<_> = result.entry(lang_name.clone()).or_default();
        for applet in std::fs::read_dir(lang.path())? {
            let applet = applet?;
            let applet_name = applet.file_name().to_string_lossy().into_owned();
            if matches!(applet_name.as_str(), ".gitignore" | "api.ts" | "node_modules") {
                continue;
            }
            if !applet.file_type()?.is_dir() {
                log::warn!("Non-directory {applet_name} in examples/{lang_name}.");
                continue;
            }
            result.insert(applet_name);
        }
    }
    Ok(result)
}

fn execute_command(command: &mut Command) -> Result<()> {
    eprintln!("{command:?}");
    let code = command.spawn()?.wait()?.code().expect("no error code");
    if code != 0 {
        std::process::exit(code);
    }
    Ok(())
}

fn replace_command(mut command: Command) -> ! {
    eprintln!("{command:?}");
    panic!("{}", command.exec());
}

fn main() -> Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("warn"));
    Flags::parse().execute()?;
    Ok(())
}
