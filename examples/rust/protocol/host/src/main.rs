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

use std::any::Any;
use std::borrow::Cow;
use std::io::Read;

use anyhow::{Context, Result, bail};
use clap::Parser;
use rusb::GlobalContext;
use wasefire_protocol::applet::{self, AppletId};
use wasefire_protocol::{
    self as service, Api, Connection as _, ConnectionExt as _, Request, Service,
};
use wasefire_protocol_usb::Connection;
use wasefire_wire::Yoke;

mod tests;

#[derive(Parser)]
struct Flags {
    #[command(flatten)]
    options: wasefire_cli_tools::action::ConnectionOptions,
    #[command(subcommand)]
    command: Command,
}

#[derive(clap::Subcommand)]
enum Command {
    /// Starts a request/response call with an applet.
    Call,

    /// Starts a tunnel with a given delimiter.
    ///
    /// The delimiter is automatically sent when standard input is closed. The tunnel is
    /// line-based.
    Tunnel { delimiter: String },

    /// Runs tests for this applet (this is not a protocol command).
    Test,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    let flags = Flags::parse();
    let connection = flags.options.connect().await?.connection() as Box<dyn Any>;
    let mut connection = match connection.downcast::<Connection<GlobalContext>>() {
        Ok(x) => *x,
        Err(_) => bail!("only usb protocol is supported"),
    };
    match flags.command {
        Command::Call => {
            let mut request = Vec::new();
            std::io::stdin().read_to_end(&mut request)?;
            let request = applet::Request { applet_id: AppletId, request: Cow::Owned(request) };
            connection.call::<service::AppletRequest>(request).await?.get();
            loop {
                let response = connection.call::<service::AppletResponse>(AppletId).await?;
                if let Some(response) = response.get() {
                    print!("{}", std::str::from_utf8(response).unwrap());
                    break Ok(());
                }
            }
        }
        Command::Tunnel { delimiter } => {
            let delimiter = delimiter.as_bytes();
            let tunnel =
                applet::Tunnel { applet_id: applet::AppletId, delimiter: Cow::Borrowed(delimiter) };
            send(&mut connection, &Api::<Request>::AppletTunnel(tunnel)).await?;
            read_tunnel(&mut connection).await?;
            for line in std::io::stdin().lines() {
                let request = line.context("reading line")?.into_bytes();
                if request == delimiter {
                    break;
                }
                connection.write(&request).await.context("sending request")?;
                let response = connection.read().await.context("receiving response")?;
                println!("{}", String::from_utf8(response.into()).unwrap());
            }
            connection.write(delimiter).await?;
            read_tunnel(&mut connection).await
        }
        Command::Test => tests::main(connection).await,
    }
}

async fn send(
    connection: &mut Connection<GlobalContext>, request: &Api<'_, Request>,
) -> Result<()> {
    connection.send(request).await.context("sending request")
}

async fn receive<T: Service>(
    connection: &mut Connection<GlobalContext>,
) -> Result<Yoke<T::Response<'static>>> {
    connection.receive::<T>().await.context("receiving response")
}

async fn read_tunnel(connection: &mut Connection<GlobalContext>) -> Result<()> {
    let () = receive::<service::AppletTunnel>(connection).await?.get();
    Ok(())
}
