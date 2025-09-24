// Copyright 2025 Google LLC
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

use std::time::Duration;

use anyhow::Result;
use clap::Parser;
use data_encoding::HEXLOWER;
use wasefire_protocol::applet::{self, AppletId};
use wasefire_protocol::{self as service, Connection, ConnectionExt as _};

#[derive(Parser)]
struct Flags {
    #[command(flatten)]
    options: wasefire_cli_tools::action::ConnectionOptions,
}

#[tokio::main]
async fn main() -> Result<()> {
    let flags = Flags::parse();
    let mut connection = flags.options.connect().await?;
    loop {
        let Stats { start, end, wait, stats } = checkpoint(&mut connection).await?;
        let delta = (end - start) as f64 / 1000000.;
        let wait = wait as f64 / 1000000.;
        let mut total = 0;
        for (addr, rssi, count) in stats {
            let addr = HEXLOWER.encode(&addr);
            let rssi = rssi / count as i32;
            println!("{addr} {rssi} x{count}");
            total += count;
        }
        let count = total as f64;
        let perf = count / delta;
        println!("{count} packets / {delta:.3} seconds ({wait:.6} waiting) = {perf:.3}");
        tokio::time::sleep(Duration::from_secs(10)).await;
    }
}

#[derive(Debug)]
struct Stats {
    start: u64,
    end: u64,
    wait: u32,
    stats: Vec<([u8; 6], i32, u32)>,
}

async fn checkpoint(connection: &mut dyn Connection) -> Result<Stats> {
    let request = applet::Request { applet_id: AppletId, request: &[] };
    connection.call::<service::AppletRequest>(request).await?.get();
    let mut yoke_response;
    let mut response = loop {
        yoke_response = connection.call::<service::AppletResponse>(AppletId).await?;
        if let Some(response) = yoke_response.get() {
            break *response;
        }
    };
    let start = u64::from_be_bytes(response[.. 8].try_into().unwrap());
    let end = u64::from_be_bytes(response[8 .. 16].try_into().unwrap());
    let wait = u32::from_be_bytes(response[16 .. 20].try_into().unwrap());
    response = &response[20 ..];
    let mut stats = Vec::new();
    while !response.is_empty() {
        let addr: [u8; 6] = response[.. 6].try_into().unwrap();
        let rssi = i32::from_be_bytes(response[6 .. 10].try_into().unwrap());
        let count = u32::from_be_bytes(response[10 .. 14].try_into().unwrap());
        stats.push((addr, rssi, count));
        response = &response[14 ..];
    }
    Ok(Stats { start, end, wait, stats })
}
