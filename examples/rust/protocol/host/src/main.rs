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

use std::io::Read;
use std::time::Duration;

use anyhow::{bail, Context, Result};
use clap::Parser;
use rpc::Connection;
use rusb::GlobalContext;
use wasefire_protocol::service::applet::{self, AppletId};
use wasefire_protocol::{Api, Request, Response};
use wasefire_protocol_usb as rpc;

mod tests;

#[derive(Parser)]
struct Flags {
    #[clap(flatten)]
    options: MainOptions,

    #[clap(subcommand)]
    command: MainCommand,
}

#[derive(clap::Args)]
struct MainOptions {
    /// Logging level.
    #[clap(long, default_value = "info")]
    log: String,
    // TODO: Add option to select device.
}

#[derive(clap::Subcommand)]
enum MainCommand {
    /// Sends a request to an applet.
    AppletRequest,

    /// Reads a response from an applet.
    AppletResponse,

    /// Runs tests for this applet (this is not a protocol command).
    Test,
}

fn main() -> Result<()> {
    let flags = Flags::parse();
    env_logger::init_from_env(env_logger::Env::new().default_filter_or(&flags.options.log));
    let candidate = rpc::choose_device().context("choosing device")?;
    let connection = candidate.connect().context("connecting to the device")?;
    match flags.command {
        MainCommand::AppletRequest => {
            let mut request = Vec::new();
            std::io::stdin().read_to_end(&mut request)?;
            let request = applet::Request { applet_id: AppletId, request: &request };
            send(&connection, &Api::<Request>::AppletRequest(request))?;
            receive(&connection, |response| Ok(matches!(response, Api::AppletRequest(()))))
        }
        MainCommand::AppletResponse => {
            send(&connection, &Api::<Request>::AppletResponse(AppletId))?;
            receive(&connection, |response| match response {
                Api::AppletResponse(applet::Response { response }) => {
                    let response = response.context("no applet response")?;
                    print!("{}", std::str::from_utf8(response).unwrap());
                    Ok(true)
                }
                _ => Ok(false),
            })
        }
        MainCommand::Test => tests::main(&connection),
    }
}

const TIMEOUT: Duration = Duration::from_secs(1);

fn send(connection: &Connection<GlobalContext>, request: &Api<Request>) -> Result<()> {
    let request = request.serialize();
    connection.send(&request, TIMEOUT).context("sending request")?;
    Ok(())
}

fn receive(
    connection: &Connection<GlobalContext>, process: impl FnOnce(&Api<Response>) -> Result<bool>,
) -> Result<()> {
    let response = connection.receive(TIMEOUT).context("receiving response")?;
    let response = Api::<Response>::deserialize(&response).context("deserializing response")?;
    if !process(&response)? {
        match response {
            Api::DeviceError(error) => bail!("error response: {error}"),
            _ => bail!("invalid response: {response:?}"),
        }
    }
    Ok(())
}
