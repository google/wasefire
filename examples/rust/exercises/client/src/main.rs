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

use std::io::{ErrorKind, Read, Write};
use std::time::Duration;

use clap::Parser;
use interface::{Request, Response};
use p256::ecdsa::signature::hazmat::PrehashVerifier;
use p256::ecdsa::{Signature, VerifyingKey};
use rand::RngCore;

#[derive(Parser)]
enum Command {
    /// Sends a Register request and stores the public key.
    Register {
        /// The private key name.
        name: String,
    },

    /// Sends an Authenticate request (with a random challenge) and verifies the signature.
    Authenticate {
        /// The private key name.
        name: String,
    },

    /// Sends a List request and prints the result.
    List,

    /// Sends a Delete request and deletes the public key.
    Delete {
        /// The private key name.
        name: String,
    },
}

fn main() {
    let request = match Command::parse() {
        Command::Register { name } => Request::Register { name },
        Command::Authenticate { name } => {
            let mut challenge = [0; 32];
            rand::thread_rng().fill_bytes(&mut challenge);
            Request::Authenticate { name, challenge }
        }
        Command::List => Request::List,
        Command::Delete { name } => Request::Delete { name },
    };
    let mut serial = connect();
    eprintln!("Sending {request:02x?}.");
    serial.write_all(&interface::serialize(&request)).unwrap();
    let response = interface::deserialize::<Result<Response, String>>(|x| match serial.read(x) {
        Err(e) if e.kind() == ErrorKind::WouldBlock => {
            panic!("Device did not reply. Is it running?");
        }
        x => x.unwrap(),
    });
    eprintln!("Received {response:02x?}.");
    let response = match response {
        Ok(x) => x,
        Err(reason) => {
            println!("Error: {reason}");
            return;
        }
    };
    match (request, response) {
        (Request::Register { name }, Response::Register { x, y }) => {
            let mut public = x.to_vec();
            public.extend_from_slice(&y);
            std::fs::write(format!("{name}.pub"), public).unwrap();
        }
        (Request::Authenticate { name, challenge }, Response::Authenticate { r, s }) => {
            let mut public = std::fs::read(format!("{name}.pub")).unwrap();
            public.insert(0, 4);
            let key = VerifyingKey::from_sec1_bytes(&public).unwrap();
            let signature = Signature::from_scalars(r, s).unwrap();
            key.verify_prehash(&challenge, &signature).unwrap();
        }
        (Request::List, Response::List { names }) => {
            for name in names {
                println!("- {name}");
            }
        }
        (Request::Delete { name }, Response::Delete) => {
            std::fs::remove_file(format!("{name}.pub")).unwrap();
        }
        _ => panic!("The response does not match the request."),
    }
}

#[cfg(feature = "usb")]
fn connect() -> Box<dyn serialport::SerialPort> {
    for info in serialport::available_ports().unwrap() {
        let path = info.port_name;
        if let serialport::SerialPortType::UsbPort(info) = info.port_type {
            if info.vid == 0x16c0 && info.pid == 0x27dd {
                let mut serial =
                    serialport::new(path, 19200).timeout(Duration::from_secs(1)).open().unwrap();
                serial.set_timeout(Duration::from_secs(10)).unwrap();
                return serial;
            }
        }
    }
    panic!("no available port");
}

#[cfg(not(feature = "usb"))]
fn connect() -> std::os::unix::net::UnixStream {
    use std::os::unix::net::UnixStream;
    let serial = UnixStream::connect("../../../../target/wasefire/uart0").unwrap();
    serial.set_read_timeout(Some(Duration::from_secs(10))).unwrap();
    serial
}
