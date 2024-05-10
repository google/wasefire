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

// DO NOT EDIT MANUALLY:
// - Edit book/src/applet/prelude/store.rs instead.
// - Then use ./scripts/sync.sh to generate this file.

#![no_std]
wasefire::applet!();

use alloc::boxed::Box;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::num::ParseIntError;
use core::ops::Range;
use core::str::FromStr;

use wasefire::usb::serial::UsbSerial;

fn main() {
    writeln(b"Usage: insert <key>[..<key>] <value>");
    writeln(b"Usage: find <key>[..<key>]");
    writeln(b"Usage: remove <key>[..<key>]");
    loop {
        // Read the command.
        let mut command = String::new();
        loop {
            serial::write_all(&UsbSerial, format!("\r\x1b[K> {command}").as_bytes()).unwrap();
            match serial::read_byte(&UsbSerial).unwrap() {
                c @ (b' ' | b'.' | b'a' ..= b'z' | b'0' ..= b'9') => command.push(c as char),
                0x7f => drop(command.pop()),
                0x0d => break,
                _ => (),
            }
        }
        serial::write_all(&UsbSerial, b"\r\n").unwrap();

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
    Insert { key: Key, value: &'a str },
    Find { key: Key },
    Remove { key: Key },
}

impl<'a> Command<'a> {
    fn parse(input: &'a str) -> Option<Self> {
        Some(match *input.split_whitespace().collect::<Vec<_>>().as_slice() {
            ["insert", key, value] => Command::Insert { key: Key::parse(key)?, value },
            ["find", key] => Command::Find { key: Key::parse(key)? },
            ["remove", key] => Command::Remove { key: Key::parse(key)? },
            _ => return None,
        })
    }

    fn process(&self) -> Result<(), Error> {
        match self {
            Command::Insert { key, value } => insert(key, value.as_bytes()),
            Command::Find { key } => {
                match find(key)? {
                    None => writeln(b"Not found."),
                    Some(value) => match core::str::from_utf8(&value) {
                        Ok(value) => writeln(format!("Found: {value}").as_bytes()),
                        Err(_) => writeln(format!("Found (not UTF-8): {value:02x?}").as_bytes()),
                    },
                }
                Ok(())
            }
            Command::Remove { key } => remove(key),
        }
    }
}

enum Key {
    Exact(usize),
    Range(Range<usize>),
}

impl FromStr for Key {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.split_once("..") {
            Some((start, end)) => Ok(Key::Range(start.parse()? .. end.parse()?)),
            None => Ok(Key::Exact(s.parse()?)),
        }
    }
}

impl Key {
    fn parse(key: &str) -> Option<Self> {
        let key: Key = key.parse().ok()?;
        let valid = match &key {
            Key::Exact(key) => *key < 4096,
            Key::Range(keys) => !keys.is_empty() && keys.end < 4096,
        };
        if !valid {
            return None;
        }
        Some(key)
    }
}

fn insert(key: &Key, value: &[u8]) -> Result<(), Error> {
    match key {
        Key::Exact(key) => store::insert(*key, value),
        Key::Range(keys) => store::fragment::insert(keys.clone(), value),
    }
}

fn find(key: &Key) -> Result<Option<Box<[u8]>>, Error> {
    match key {
        Key::Exact(key) => store::find(*key),
        Key::Range(keys) => store::fragment::find(keys.clone()),
    }
}

fn remove(key: &Key) -> Result<(), Error> {
    match key {
        Key::Exact(key) => store::remove(*key),
        Key::Range(keys) => store::fragment::remove(keys.clone()),
    }
}

fn writeln(buf: &[u8]) {
    serial::write_all(&UsbSerial, buf).unwrap();
    serial::write_all(&UsbSerial, b"\r\n").unwrap();
}
