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

#![feature(try_find)]

use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::Duration;

use anyhow::{bail, Result};
use clap::{CommandFactory, Parser, ValueHint};
use clap_complete::Shell;
use data_encoding::HEXLOWER_PERMISSIVE as HEX;
use rusb::GlobalContext;
use wasefire_cli_tools::{action, fs};
use wasefire_protocol::{self as service, platform};
use wasefire_protocol_usb::{Candidate, Connection};

#[derive(Parser)]
#[command(version, about)]
struct Flags {
    #[command(flatten)]
    options: Options,

    #[command(subcommand)]
    action: Action,
}

#[derive(clap::Args)]
struct Options {
    /// Serial of the platform to connect to.
    #[arg(long, env = "WASEFIRE_SERIAL")]
    serial: Option<String>,
}

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

    AppletRpc(action::AppletRpc),

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
    CONNECTION.lock().unwrap().set(flags.options.serial);
    let dir = std::env::current_dir()?;
    match flags.action {
        Action::AppletList => bail!("not implemented yet"),
        Action::AppletInstall => bail!("not implemented yet"),
        Action::AppletUpdate => bail!("not implemented yet"),
        Action::AppletUninstall => bail!("not implemented yet"),
        Action::AppletRpc(x) => x.run(CONNECTION.lock().unwrap().get()?),
        Action::PlatformList => platform_list(),
        Action::PlatformUpdate => bail!("not implemented yet"),
        Action::RustAppletNew(x) => x.run(),
        Action::RustAppletBuild(x) => x.run(dir),
        Action::RustAppletTest(x) => x.run(dir),
        Action::Completion(x) => x.run(),
    }
}

enum GlobalConnection {
    Invalid,
    Ready { serial: Option<String> },
    Connected { connection: Connection<GlobalContext> },
}

impl GlobalConnection {
    fn set(&mut self, serial: Option<String>) {
        match self {
            GlobalConnection::Invalid => *self = GlobalConnection::Ready { serial },
            _ => unreachable!(),
        }
    }

    fn get(&mut self) -> Result<&Connection<GlobalContext>> {
        if let GlobalConnection::Ready { serial } = self {
            *self = GlobalConnection::Connected { connection: connect(serial.as_deref())? };
        }
        match self {
            GlobalConnection::Connected { connection } => Ok(connection),
            _ => unreachable!(),
        }
    }
}

static CONNECTION: Mutex<GlobalConnection> = Mutex::new(GlobalConnection::Invalid);

fn platform_list() -> Result<()> {
    let context = GlobalContext::default();
    let candidates = wasefire_protocol_usb::list(&context)?;
    println!("There are {} connected platforms:", candidates.len());
    for candidate in candidates {
        let connection = candidate.clone().connect()?;
        let info = connection.call::<service::PlatformInfo>((), TIMEOUT)?;
        let platform::Info { serial, version } = info.get();
        let serial = HEX.encode(serial);
        let version = HEX.encode(version);
        println!("- serial={serial} version={version}");
    }
    Ok(())
}

fn connect(serial: Option<&str>) -> Result<Connection<GlobalContext>> {
    let context = GlobalContext::default();
    let mut candidates = wasefire_protocol_usb::list(&context)?;
    let candidate = match (serial, candidates.len()) {
        (None, 0) => bail!("no connected platforms"),
        (None, 1) => candidates.pop().unwrap(),
        (None, n) => {
            eprintln!("Choose one of the {n} connected platforms using its --serial option:");
            for candidate in candidates {
                eprintln!("    --serial={}", get_serial(&candidate)?);
            }
            bail!("more than one connected platform");
        }
        (Some(serial), _) => {
            match candidates.into_iter().try_find(|x| anyhow::Ok(get_serial(x)? == serial))? {
                Some(x) => x,
                None => bail!("no connected platform with serial={serial}"),
            }
        }
    };
    Ok(candidate.connect()?)
}

fn get_serial(candidate: &Candidate<GlobalContext>) -> Result<String> {
    let connection = candidate.clone().connect()?;
    let info = connection.call::<service::PlatformInfo>((), TIMEOUT)?;
    Ok(HEX.encode(info.get().serial))
}

const TIMEOUT: Duration = Duration::from_secs(1);
