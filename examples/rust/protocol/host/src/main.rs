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

use anyhow::{Context, Result};
use clap::Parser;
use rusb::GlobalContext;
use wasefire_protocol::applet::{self, AppletId};
use wasefire_protocol::{self as service, Api, Connection as _, ConnectionExt, Request, Service};
use wasefire_protocol_usb::{self as rpc, Connection};
use wasefire_wire::Yoke;

mod tests;

#[derive(Parser)]
enum Flags {
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
    let context = GlobalContext::default();
    let candidate = rpc::choose_device(&context).context("choosing device")?;
    let mut connection =
        candidate.connect(Duration::from_secs(1)).context("connecting to the device")?;
    match flags {
        Flags::Call => {
            let mut request = Vec::new();
            std::io::stdin().read_to_end(&mut request)?;
            let request = applet::Request { applet_id: AppletId, request: &request };
            connection.call::<service::AppletRequest>(request).await?.get();
            loop {
                let response = connection.call::<service::AppletResponse>(AppletId).await?;
                if let Some(response) = response.get() {
                    print!("{}", std::str::from_utf8(response).unwrap());
                    break Ok(());
                }
            }
        }
        Flags::Tunnel { delimiter } => {
            let delimiter = delimiter.as_bytes();
            let tunnel = applet::Tunnel { applet_id: applet::AppletId, delimiter };
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
        Flags::Test => tests::main(connection).await,
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
