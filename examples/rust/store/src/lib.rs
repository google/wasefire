// Copyright 2022 Google LLC
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

//! Demonstrates simple store usage.
//!
//! The applet additionally relies on USB serial. It describes its own usage when connecting on the
//! USB serial.

#![no_std]

extern crate alloc;

use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use wasefire::*;

#[no_mangle]
pub extern "C" fn main() {
    writeln(b"Usage: insert <key> <value>");
    writeln(b"Usage: find <key>");
    writeln(b"Usage: remove <key>");
    loop {
        // Read the command.
        let mut command = String::new();
        loop {
            usb::serial::write_all(format!("\r\x1b[K> {command}").as_bytes()).unwrap();
            match usb::serial::read_byte().unwrap() {
                c @ (b' ' | b'a' ..= b'z' | b'0' ..= b'9') => command.push(c as char),
                0x7f => drop(command.pop()),
                0x0d => break,
                _ => (),
            }
        }
        usb::serial::write_all(b"\r\n").unwrap();

        // Parse the command.
        let command = match Command::parse(&command) {
            Some(x) => x,
            None => {
                writeln(b"Failed: InvalidCommand");
                continue;
            }
        };

        // Process the command.
        if let Err(error) = command.process() {
            writeln(format!("Failed: {error:?}").as_bytes());
        }
    }
}

enum Command<'a> {
    Insert { key: usize, value: &'a str },
    Find { key: usize },
    Remove { key: usize },
}

impl<'a> Command<'a> {
    fn parse(input: &'a str) -> Option<Self> {
        Some(match input.split_whitespace().collect::<Vec<_>>().as_slice() {
            &["insert", key, value] => Command::Insert { key: key.parse().ok()?, value },
            &["find", key] => Command::Find { key: key.parse().ok()? },
            &["remove", key] => Command::Remove { key: key.parse().ok()? },
            _ => return None,
        })
    }

    fn process(&self) -> Result<(), store::Error> {
        match self {
            Command::Insert { key, value } => store::insert(*key, value.as_bytes()),
            Command::Find { key } => {
                match store::find(*key)? {
                    None => writeln(b"Not found."),
                    Some(value) => match core::str::from_utf8(&value) {
                        Ok(value) => writeln(format!("Found: {value}").as_bytes()),
                        Err(_) => writeln(format!("Found (not UTF-8): {value:02x?}").as_bytes()),
                    },
                }
                Ok(())
            }
            Command::Remove { key } => store::remove(*key),
        }
    }
}

fn writeln(buf: &[u8]) {
    usb::serial::write_all(buf).unwrap();
    usb::serial::write_all(b"\r\n").unwrap();
}
