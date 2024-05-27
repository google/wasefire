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
use std::process::Command;

use anyhow::{ensure, Result};
use cargo_metadata::{Metadata, MetadataCommand};
use clap::{ValueEnum, ValueHint};

use crate::{cmd, fs};

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
    pub fn run(&self) -> Result<()> {
        let RustAppletNew { path, name } = self;
        let mut cargo = Command::new("cargo");
        cargo.args(["new", "--lib"]).arg(path);
        if let Some(name) = name {
            cargo.arg(format!("--name={name}"));
        }
        cmd::execute(&mut cargo)?;
        cmd::execute(Command::new("cargo").args(["add", "wasefire"]).current_dir(path))?;
        let mut cargo = Command::new("cargo");
        cargo.args(["add", "wasefire-stub", "--optional"]);
        cmd::execute(cargo.current_dir(path))?;
        let mut sed = Command::new("sed");
        sed.arg("-i");
        sed.arg("s#^wasefire-stub\\( = .\"dep:wasefire-stub\"\\)#test\\1, \"wasefire/test\"#");
        sed.arg("Cargo.toml");
        cmd::execute(sed.current_dir(path))?;
        std::fs::remove_file(path.join("src/lib.rs"))?;
        fs::write(path.join("src/lib.rs"), include_str!("data/lib.rs"))?;
        Ok(())
    }
}

/// Builds a Rust applet from its project.
#[derive(Default, clap::Args)]
pub struct RustAppletBuild {
    /// Builds for production, disabling debugging facilities.
    #[arg(long)]
    pub prod: bool,

    /// Builds a native applet, e.g. --native=thumbv7em-none-eabi.
    #[arg(long, value_name = "TARGET")]
    pub native: Option<String>,

    /// Copies the final artifacts to this directory instead of target/wasefire.
    #[arg(long, value_name = "DIR", value_hint = ValueHint::DirPath)]
    pub output: Option<PathBuf>,

    /// Cargo profile, defaults to release.
    #[arg(long)]
    pub profile: Option<String>,

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
    pub fn run(&self, dir: impl AsRef<Path>) -> Result<()> {
        let metadata = metadata(dir.as_ref())?;
        let package = &metadata.packages[0];
        let target_dir = fs::try_relative(std::env::current_dir()?, &metadata.target_directory)?;
        let name = package.name.replace('-', "_");
        let mut cargo = Command::new("cargo");
        let mut rustflags = vec![
            "-C panic=abort".to_string(),
            "-C codegen-units=1".to_string(),
            "-C embed-bitcode=yes".to_string(),
            "-C lto=fat".to_string(),
        ];
        cargo.args(["rustc", "--lib"]);
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
        match &self.profile {
            Some(profile) => drop(cargo.arg(format!("--profile={profile}"))),
            None => drop(cargo.arg("--release")),
        }
        if let Some(level) = self.opt_level {
            rustflags.push(format!("-C opt-level={level}"));
        }
        cargo.args(&self.cargo);
        if self.prod {
            cargo.args(["-Zbuild-std=core,alloc", "-Zbuild-std-features=panic_immediate_abort"]);
        } else {
            cargo.env("WASEFIRE_DEBUG", "");
        }
        cargo.env("RUSTFLAGS", rustflags.join(" "));
        cargo.current_dir(dir);
        cmd::execute(&mut cargo)?;
        let out_dir = match &self.output {
            Some(x) => x.clone(),
            None => "target/wasefire".into(),
        };
        let (src, dst) = match &self.native {
            None => (format!("wasm32-unknown-unknown/release/{name}.wasm"), "applet.wasm"),
            Some(target) => (format!("{target}/release/lib{name}.a"), "libapplet.a"),
        };
        let applet = out_dir.join(dst);
        if fs::copy_if_changed(target_dir.join(src), &applet)? && dst.ends_with(".wasm") {
            optimize_wasm(&applet, self.opt_level)?;
        }
        Ok(())
    }
}

/// Runs the unit-tests of a Rust applet project.
#[derive(clap::Args)]
pub struct RustAppletTest {
    /// Extra arguments to cargo, e.g. --features=foo.
    #[clap(last = true)]
    cargo: Vec<String>,
}

impl RustAppletTest {
    pub fn run(&self, dir: impl AsRef<Path>) -> Result<()> {
        let metadata = metadata(dir.as_ref())?;
        let package = &metadata.packages[0];
        ensure!(package.features.contains_key("test"), "missing test feature");
        let mut cargo = Command::new("cargo");
        cargo.args(["test", "--features=test"]);
        cargo.args(&self.cargo);
        cmd::replace(cargo)
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

impl Display for OptLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_possible_value().unwrap().get_name())
    }
}

/// Strips and optimizes a WASM applet.
pub fn optimize_wasm(applet: impl AsRef<Path>, opt_level: Option<OptLevel>) -> Result<()> {
    let mut strip = Command::new("wasm-strip");
    strip.arg(applet.as_ref());
    cmd::execute(&mut strip)?;
    let mut opt = Command::new("wasm-opt");
    opt.args(["--enable-bulk-memory", "--enable-sign-ext", "--enable-mutable-globals"]);
    match opt_level {
        Some(level) => drop(opt.arg(format!("-O{level}"))),
        None => drop(opt.arg("-O")),
    }
    opt.arg(applet.as_ref());
    opt.arg("-o");
    opt.arg(applet.as_ref());
    cmd::execute(&mut opt)?;
    Ok(())
}

fn metadata(dir: impl Into<PathBuf>) -> Result<Metadata> {
    let metadata = MetadataCommand::new().current_dir(dir).no_deps().exec()?;
    ensure!(metadata.packages.len() == 1, "not exactly one package");
    Ok(metadata)
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
