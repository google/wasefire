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

#![feature(never_type)]

use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use clap::{CommandFactory, Parser, ValueHint};
use clap_complete::Shell;
use tokio::process::Command;
use wasefire_cli_tools::{action, cmd, fs};

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

    /// Starts a host platform.
    Host(Host),

    #[group(id = "Action::PlatformInfo")]
    PlatformInfo {
        #[command(flatten)]
        options: action::ConnectionOptions,
        #[command(flatten)]
        action: action::PlatformInfo,
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
struct Host {
    /// Path of the directory containing the host platform files.
    ///
    /// Such a directory may contain:
    /// - `applet.bin` the persistent applet
    /// - `platform.bin` the platform code
    /// - `storage.bin` the persistent storage
    /// - `uart0` the UNIX socket for the UART
    /// - `web` the web interface assets
    ///
    /// If the platform code is missing (including if the directory does not exist), a default
    /// platform code is created and started.
    #[arg(long, default_value = "wasefire/host", value_hint = ValueHint::DirPath)]
    dir: PathBuf,

    /// Arguments to forward to the runner.
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    args: Vec<String>,
}

#[derive(clap::Args)]
struct Completion {
    /// Generates a completion file for this shell (tries to guess by default).
    shell: Option<Shell>,

    /// Where to generate the completion file.
    #[arg(long, default_value = "-", value_hint = ValueHint::FilePath)]
    output: PathBuf,
}

impl Host {
    async fn run(&self) -> Result<!> {
        let bin = self.dir.join("platform.bin");
        #[cfg(feature = "_dev")]
        if !fs::exists(&bin).await {
            let bundle = "target/wasefire/platform.bin";
            anyhow::ensure!(
                fs::exists(&bundle).await,
                "Run `cargo xtask runner host bundle` first"
            );
            fs::copy(bundle, &bin).await?;
        }
        #[cfg(not(feature = "_dev"))]
        if !fs::exists(&bin).await {
            fs::create_dir_all(&self.dir).await?;
            static HOST_PLATFORM: &[u8] = include_bytes!(env!("WASEFIRE_HOST_PLATFORM"));
            let mut params = fs::WriteParams::new(&bin);
            params.options().write(true).create_new(true).mode(0o777);
            fs::write(params, HOST_PLATFORM).await?;
        }
        loop {
            let mut host = Command::new(&bin);
            host.arg(&self.dir);
            host.args(&self.args);
            let code = cmd::spawn(&mut host)?.wait().await?.code().context("no error code")?;
            if code != 0 {
                std::process::exit(code);
            }
        }
    }
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
    env_logger::init();
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
        Action::Host(x) => x.run().await?,
        Action::PlatformInfo { options, action } => action.run(&mut options.connect().await?).await,
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
