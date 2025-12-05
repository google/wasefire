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

use std::borrow::Cow;
use std::hash::{DefaultHasher, Hash as _, Hasher as _};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;

use anyhow::Result;
use rusb::{Error, GlobalContext};
use wasefire_protocol::{Api, Connection as _, Request, applet};
use wasefire_protocol_usb::Connection;

pub(crate) async fn main(mut connection: Connection<GlobalContext>) -> Result<()> {
    connection.set_timeout(Duration::from_millis(200));
    let connection = &mut connection;
    println!("Enter tunnel.");
    let tunnel = applet::Tunnel { applet_id: applet::AppletId, delimiter: Cow::Borrowed(b"EOF") };
    crate::send(connection, &Api::<Request>::AppletTunnel(tunnel)).await?;
    crate::read_tunnel(connection).await?;

    println!("Round-trip:");
    for len in [0, 1, 62, 63, 64, 125, 126, 127, 188, 189] {
        print!("- {len}:");
        let packet: Vec<u8> = (b'1' ..= b'9').cycle().take(len).collect();
        connection.write(&packet).await.unwrap();
        assert_eq!(connection.read().await.unwrap()[..], packet[..]);
        println!(" ok");
    }

    println!("Not reading a response should not jam (a warning is expected).");
    connection.write(b"hello").await.unwrap();
    ping_pong(connection).await;

    println!("Short and invalid packets:");
    for len in [0, 1, 63, 64, 65, 128, 155] {
        let packet = vec![0xc3; len];
        print!("- [0; {len}]");
        write(connection, &packet);
        ping_pong(connection).await;
        print!(" ping-pong");
        write(connection, &packet);
        read_timeout(connection);
        println!(" timeout");
    }

    println!("Close tunnel.");
    connection.write(b"EOF").await?;
    crate::read_tunnel(connection).await?;
    Ok(())
}

fn write(connection: &Connection<GlobalContext>, packet: &[u8]) {
    assert_eq!(connection.testonly_write(packet), Ok(packet.len()));
}

fn read_timeout(connection: &Connection<GlobalContext>) {
    let mut packet = [0; 64];
    assert_eq!(connection.testonly_read(&mut packet), Err(Error::Timeout));
}

async fn ping_pong(connection: &mut Connection<GlobalContext>) {
    static ID: AtomicU32 = AtomicU32::new(0);
    let id = {
        // This is just a way to have deterministic randomness.
        let mut hasher = DefaultHasher::new();
        ID.fetch_add(1, Ordering::Relaxed).hash(&mut hasher);
        hasher.finish()
    };
    connection.write(format!("ping {id}.").as_bytes()).await.unwrap();
    assert_eq!(connection.read().await.unwrap().as_ref(), format!("PONG {id}.").as_bytes());
}
