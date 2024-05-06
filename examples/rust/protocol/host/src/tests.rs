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

use std::time::Duration;

use anyhow::Result;
use rusb::{Error, GlobalContext};
use wasefire_protocol::service::applet;
use wasefire_protocol::{Api, Request};
use wasefire_protocol_usb::Connection;

pub fn main(connection: &Connection<GlobalContext>) -> Result<()> {
    println!("Enter tunnel.");
    let tunnel = applet::Tunnel { applet_id: applet::AppletId, delimiter: b"EOF" };
    crate::send(connection, &Api::<Request>::AppletTunnel(tunnel))?;
    crate::read_tunnel(connection)?;

    println!("Round-trip:");
    for len in [0, 1, 62, 63, 64, 125, 126, 127, 188, 189] {
        print!("- {len}:");
        let packet: Vec<u8> = (b'1' ..= b'9').cycle().take(len).collect();
        connection.send(&packet, TIMEOUT).unwrap();
        assert_eq!(connection.receive(TIMEOUT).unwrap(), packet);
        println!(" ok");
    }

    println!("Not reading a response should not jam (a warning is expected).");
    connection.send(b"hello", TIMEOUT).unwrap();
    ping_pong(connection);

    println!("Short and invalid packets:");
    for len in [0, 1, 63, 64, 65, 128, 155] {
        let packet = vec![0xc3; len];
        print!("- [0; {len}]");
        write(connection, &packet);
        ping_pong(connection);
        print!(" ping-pong");
        write(connection, &packet);
        read_timeout(connection);
        println!(" timeout");
    }

    println!("Close tunnel.");
    connection.send(b"EOF", TIMEOUT)?;
    crate::read_tunnel(connection)?;
    Ok(())
}

fn write(connection: &Connection<GlobalContext>, packet: &[u8]) {
    assert_eq!(connection.testonly_write(packet, TIMEOUT), Ok(packet.len()));
}

fn read_timeout(connection: &Connection<GlobalContext>) {
    let mut packet = [0; 64];
    assert_eq!(connection.testonly_read(&mut packet, TIMEOUT), Err(Error::Timeout));
}

fn ping_pong(connection: &Connection<GlobalContext>) {
    connection.send(b"ping", TIMEOUT).unwrap();
    assert_eq!(connection.receive(TIMEOUT).unwrap(), b"PONG");
}

const TIMEOUT: Duration = Duration::from_millis(200);
