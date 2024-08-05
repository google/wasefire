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

use anyhow::{bail, Result};
use clap::{CommandFactory, Parser, ValueHint};
use clap_complete::Shell;
use wasefire_cli_tools::{action, fs};

#[derive(Parser)]
#[command(name = "wasefire", version, about)]
struct Flags {
    #[command(flatten)]
    options: Options,

    #[command(subcommand)]
    action: Action,
}

#[test]
fn flags() {
    <Flags as clap::CommandFactory>::command().debug_assert();
}

#[derive(clap::Args)]
struct Options {}

#[derive(clap::Subcommand)]
enum Action {
    /// Lists the applets installed on a platform.
    AppletList,

    #[group(id = "Action::AppletInstall")]
    AppletInstall {
        #[command(flatten)]
        options: action::ConnectionOptions,
        #[command(flatten)]
        action: action::AppletInstall,
        #[command(subcommand)]
        command: Option<AppletInstallCommand>,
    },

    /// Updates an applet on a platform.
    AppletUpdate,

    #[group(id = "Action::AppletUninstall")]
    AppletUninstall {
        #[command(flatten)]
        options: action::ConnectionOptions,
        #[command(flatten)]
        action: action::AppletUninstall,
    },

    #[group(id = "Action::AppletExitStatus")]
    AppletExitStatus {
        #[command(flatten)]
        options: action::ConnectionOptions,
        #[command(flatten)]
        action: action::AppletExitStatus,
    },

    // TODO(https://github.com/clap-rs/clap/issues/2621): We should be able to remove the explicit
    // group name.
    #[group(id = "Action::AppletRpc")]
    AppletRpc {
        #[command(flatten)]
        options: action::ConnectionOptions,
        #[command(flatten)]
        action: action::AppletRpc,
    },
    PlatformList(action::PlatformList),

    /// Prints the platform update metadata (possibly binary output).
    PlatformUpdateMetadata {
        #[command(flatten)]
        options: action::ConnectionOptions,
    },

    PlatformUpdateTransfer {
        #[command(flatten)]
        options: action::ConnectionOptions,
        #[command(flatten)]
        action: action::PlatformUpdate,
    },

    #[group(id = "Action::PlatformReboot")]
    PlatformReboot {
        #[command(flatten)]
        options: action::ConnectionOptions,
        #[command(flatten)]
        action: action::PlatformReboot,
    },
    #[group(id = "Action::PlatformLock")]
    PlatformLock {
        #[command(flatten)]
        options: action::ConnectionOptions,
        #[command(flatten)]
        action: action::PlatformLock,
    },
    #[group(id = "Action::PlatformRpc")]
    PlatformRpc {
        #[command(flatten)]
        options: action::ConnectionOptions,
        #[command(flatten)]
        action: action::PlatformRpc,
    },
    RustAppletNew(action::RustAppletNew),
    RustAppletBuild(action::RustAppletBuild),
    RustAppletTest(action::RustAppletTest),

    /// Generates a shell completion file.
    Completion(Completion),
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
struct Completion {
    /// Generates a completion file for this shell (tries to guess by default).
    shell: Option<Shell>,

    /// Where to generate the completion file.
    #[arg(long, default_value = "-", value_hint = ValueHint::FilePath)]
    output: PathBuf,
}

impl Completion {
    async fn run(&self) -> Result<()> {
        let shell = match self.shell.or_else(Shell::from_env) {
            Some(x) => x,
            None => bail!("failed to guess a shell"),
        };
        let mut cmd = Flags::command();
        let name = "wasefire".to_string();
        let mut output: Box<dyn Write> = if self.output == Path::new("-") {
            Box::new(std::io::stdout())
        } else {
            fs::create_parent(&self.output).await?;
            Box::new(File::create(&self.output)?)
        };
        clap_complete::generate(shell, &mut cmd, name, &mut output);
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let flags = Flags::parse();
    let dir = std::env::current_dir()?;
    match flags.action {
        Action::AppletList => bail!("not implemented yet"),
        Action::AppletInstall { options, action, command } => {
            let mut connection = options.connect().await?;
            action.run(&mut connection).await?;
            match command {
                None => Ok(()),
                Some(AppletInstallCommand::Wait { mut action }) => {
                    action.wait.ensure_wait();
                    action.run(&mut connection).await
                }
            }
        }
        Action::AppletUpdate => bail!("not implemented yet"),
        Action::AppletUninstall { options, action } => {
            action.run(&mut options.connect().await?).await
        }
        Action::AppletExitStatus { options, action } => {
            action.run(&mut options.connect().await?).await
        }
        Action::AppletRpc { options, action } => action.run(&mut options.connect().await?).await,
        Action::PlatformList(x) => x.run().await,
        Action::PlatformUpdateMetadata { options } => {
            let metadata = action::PlatformUpdate::metadata(&mut options.connect().await?).await?;
            fs::write_stdout(metadata.get()).await
        }
        Action::PlatformUpdateTransfer { options, action } => {
            action.run(&mut options.connect().await?).await
        }
        Action::PlatformReboot { options, action } => {
            action.run(&mut options.connect().await?).await
        }
        Action::PlatformLock { options, action } => action.run(&mut options.connect().await?).await,
        Action::PlatformRpc { options, action } => action.run(&mut options.connect().await?).await,
        Action::RustAppletNew(x) => x.run().await,
        Action::RustAppletBuild(x) => x.run(dir).await,
        Action::RustAppletTest(x) => x.run(dir).await,
        Action::Completion(x) => x.run().await,
    }
}
