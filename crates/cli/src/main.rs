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
        Action::AppletList
        | Action::AppletInstall
        | Action::AppletUpdate
        | Action::AppletUninstall => bail!("not implemented yet (depends on #56)"),
        Action::PlatformList | Action::PlatformUpdate => bail!("not implemented yet"),
        Action::RustAppletNew(x) => x.run(),
        Action::RustAppletBuild(x) => x.run(dir),
        Action::RustAppletTest(x) => x.run(dir),
        Action::Completion(x) => x.run(),
    }
}
