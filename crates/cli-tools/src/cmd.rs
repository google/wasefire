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

//! Helpers around `tokio::process::Command`.

use std::os::unix::process::CommandExt;
use std::process::Output;

use anyhow::{ensure, Context, Result};
use tokio::process::{Child, Command};

/// Spawns a command.
pub fn spawn(command: &mut Command) -> Result<Child> {
    debug!("{:?}", command.as_std());
    Ok(command.spawn()?)
}

/// Executes a command making sure it's successful.
pub async fn execute(command: &mut Command) -> Result<()> {
    let code = spawn(command)?.wait().await?.code().context("no error code")?;
    ensure!(code == 0, "failed with code {code}");
    Ok(())
}

/// Replaces the current program with the command.
pub fn replace(mut command: Command) -> ! {
    debug!("{:?}", command.as_std());
    panic!("{}", command.as_std_mut().exec());
}

/// Executes the command making sure it's successful and returns its output.
pub async fn output(command: &mut Command) -> Result<Output> {
    debug!("{:?}", command.as_std());
    let output = command.output().await?;
    ensure!(output.status.success(), "failed with status {}", output.status);
    Ok(output)
}

/// Executes the command making sure it's successful and returns exactly one line.
pub async fn output_line(command: &mut Command) -> Result<String> {
    let mut output = output(command).await?;
    assert!(output.stderr.is_empty());
    assert_eq!(output.stdout.pop(), Some(b'\n'));
    Ok(String::from_utf8(output.stdout)?)
}
