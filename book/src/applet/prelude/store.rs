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

//{ ANCHOR: all
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

//{ ANCHOR: usage
fn main() {
    writeln(b"Usage: insert <key>[..<key>] <value>");
    writeln(b"Usage: find <key>[..<key>]");
    writeln(b"Usage: remove <key>[..<key>]");
    //} ANCHOR_END: usage
    //{ ANCHOR: loop
    loop {
        //} ANCHOR_END: loop
        //{ ANCHOR: read
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
        //} ANCHOR_END: read

        //{ ANCHOR: parse
        // Parse the command.
        let command = match Command::parse(&command) {
            Some(x) => x,
            None => {
                writeln(b"Failed: InvalidCommand");
                continue;
            }
        };
        //} ANCHOR_END: parse

        //{ ANCHOR: process
        // Process the command.
        if let Err(error) = command.process() {
            writeln(format!("Failed: {error:?}").as_bytes());
        }
        //} ANCHOR_END: process
    }
}

//{ ANCHOR: command
enum Command<'a> {
    Insert { key: Key, value: &'a str },
    Find { key: Key },
    Remove { key: Key },
}
//} ANCHOR_END: command

//{ ANCHOR: impl_parse
impl<'a> Command<'a> {
    fn parse(input: &'a str) -> Option<Self> {
        Some(match *input.split_whitespace().collect::<Vec<_>>().as_slice() {
            ["insert", key, value] => Command::Insert { key: Key::parse(key)?, value },
            ["find", key] => Command::Find { key: Key::parse(key)? },
            ["remove", key] => Command::Remove { key: Key::parse(key)? },
            _ => return None,
        })
    }
    //} ANCHOR_END: impl_parse

    //{ ANCHOR: process_signature
    fn process(&self) -> Result<(), store::Error> {
        //} ANCHOR_END: process_signature
        //{ ANCHOR: process_insert
        match self {
            Command::Insert { key, value } => insert(key, value.as_bytes()),
            //} ANCHOR_END: process_insert
            //{ ANCHOR: process_find
            Command::Find { key } => {
                match find(key)? {
                    //} ANCHOR_END: process_find
                    //{ ANCHOR: process_none
                    None => writeln(b"Not found."),
                    //} ANCHOR_END: process_none
                    //{ ANCHOR: process_ok
                    Some(value) => match core::str::from_utf8(&value) {
                        Ok(value) => writeln(format!("Found: {value}").as_bytes()),
                        //} ANCHOR_END: process_ok
                        //{ ANCHOR: process_err
                        Err(_) => writeln(format!("Found (not UTF-8): {value:02x?}").as_bytes()),
                        //} ANCHOR_END: process_err
                    },
                }
                Ok(())
            }
            //{ ANCHOR: process_remove
            Command::Remove { key } => remove(key),
            //} ANCHOR_END: process_remove
        }
    }
}

//{ ANCHOR: key
enum Key {
    Exact(usize),
    Range(Range<usize>),
}
//} ANCHOR_END: key

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

//{ ANCHOR: helpers
fn insert(key: &Key, value: &[u8]) -> Result<(), store::Error> {
    match key {
        Key::Exact(key) => store::insert(*key, value),
        Key::Range(keys) => store::fragment::insert(keys.clone(), value),
    }
}

fn find(key: &Key) -> Result<Option<Box<[u8]>>, store::Error> {
    match key {
        Key::Exact(key) => store::find(*key),
        Key::Range(keys) => store::fragment::find(keys.clone()),
    }
}

fn remove(key: &Key) -> Result<(), store::Error> {
    match key {
        Key::Exact(key) => store::remove(*key),
        Key::Range(keys) => store::fragment::remove(keys.clone()),
    }
}
//} ANCHOR_END: helpers

//{ ANCHOR: writeln
fn writeln(buf: &[u8]) {
    serial::write_all(&UsbSerial, buf).unwrap();
    serial::write_all(&UsbSerial, b"\r\n").unwrap();
}
//} ANCHOR_END: writeln
//} ANCHOR_END: all
