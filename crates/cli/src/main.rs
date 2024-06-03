// Copyright 2023 Google LLC
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

use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{bail, Result};
use clap::{CommandFactory, Parser, ValueHint};
use clap_complete::Shell;
use data_encoding::HEXLOWER_PERMISSIVE as HEX;
use wasefire_cli_tools::{action, fs};
use wasefire_protocol::{self as service, platform};

#[derive(Parser)]
#[command(version, about)]
struct Flags {
    #[command(flatten)]
    options: Options,

    #[command(subcommand)]
    action: Action,
}

#[derive(clap::Args)]
struct Options {}

#[derive(clap::Subcommand)]
enum Action {
    /// Lists the applets installed on a platform.
    AppletList,

    /// Installs an applet on a platform.
    AppletInstall,

    /// Updates an applet on a platform.
    AppletUpdate,

    /// Uninstalls an applet from a platform.
    AppletUninstall,

    /// Lists the connected platforms.
    PlatformList,

    /// Updates a connected platform.
    PlatformUpdate,

    RustAppletNew(action::RustAppletNew),
    RustAppletBuild(action::RustAppletBuild),
    RustAppletTest(action::RustAppletTest),

    /// Generates a shell completion file.
    Completion(Completion),
}

#[derive(clap::Args)]
struct Completion {
    /// Generates a completion file for this shell (tries to guess by default).
    shell: Option<Shell>,

    /// Where to generate the completion file.
    #[arg(long, default_value = "-", value_hint = ValueHint::FilePath)]
    output: PathBuf,
}

impl Options {
    fn run(&self) -> Result<()> {
        Ok(())
    }
}

impl Completion {
    fn run(&self) -> Result<()> {
        let shell = match self.shell.or_else(Shell::from_env) {
            Some(x) => x,
            None => bail!("failed to guess a shell"),
        };
        let mut cmd = Flags::command();
        let name = "wasefire".to_string();
        let mut output: Box<dyn Write> = if self.output == Path::new("-") {
            Box::new(std::io::stdout())
        } else {
            fs::create_parent(&self.output)?;
            Box::new(File::create(&self.output)?)
        };
        clap_complete::generate(shell, &mut cmd, name, &mut output);
        Ok(())
    }
}

fn main() -> Result<()> {
    let flags = Flags::parse();
    flags.options.run()?;
    let dir = std::env::current_dir()?;
    match flags.action {
        Action::AppletList => bail!("not implemented yet"),
        Action::AppletInstall => bail!("not implemented yet"),
        Action::AppletUpdate => bail!("not implemented yet"),
        Action::AppletUninstall => bail!("not implemented yet"),
        Action::PlatformList => platform_list(),
        Action::PlatformUpdate => bail!("not implemented yet"),
        Action::RustAppletNew(x) => x.run(),
        Action::RustAppletBuild(x) => x.run(dir),
        Action::RustAppletTest(x) => x.run(dir),
        Action::Completion(x) => x.run(),
    }
}

fn platform_list() -> Result<()> {
    let context = wasefire_protocol_usb::GlobalContext::default();
    let candidates = wasefire_protocol_usb::list(&context)?;
    println!("There are {} connected platforms:", candidates.len());
    for candidate in candidates {
        let connection = candidate.clone().connect()?;
        let info = connection.call::<service::PlatformInfo>((), TIMEOUT)?;
        let platform::Info { serial, version } = info.get();
        let serial = HEX.encode(serial);
        let version = HEX.encode(version);
        println!("- serial:{serial} version:{version}");
    }
    Ok(())
}

const TIMEOUT: Duration = Duration::from_secs(1);
