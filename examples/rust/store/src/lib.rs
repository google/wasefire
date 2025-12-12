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
//! The applet uses the platform protocol applet RPC system for its interface. It is a simple
//! text-based interface and describes its usage when receiving "help" as a request:
//!
//!     ( echo help; cat ) | wasefire applet-rpc --repl

// DO NOT EDIT MANUALLY:
// - Edit book/src/applet/prelude/store.rs instead.
// - Then use ./scripts/sync.sh to generate this file.

#![no_std]
#![feature(try_blocks)]
#![feature(try_blocks_heterogeneous)]
wasefire::applet!();

use alloc::boxed::Box;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::num::ParseIntError;
use core::ops::Range;
use core::str::FromStr;

fn main() {
    rpc::Listener::new(&platform::protocol::RpcProtocol, handler).leak();
}

fn handler(request: Vec<u8>) -> Vec<u8> {
    // Parse and process the request.
    let result = try bikeshed Result<String, String> {
        let request = String::from_utf8(request).map_err(|_| "Request is not UTF-8")?;
        Command::parse(&request)?.process()?
    };
    // Format output including error and next prompt.
    let mut output = result.unwrap_or_else(|error| format!("Error: {error}"));
    output.push_str("\n> ");
    output.into_bytes()
}

enum Command<'a> {
    Help,
    Insert { key: Key, value: &'a str },
    Find { key: Key },
    Remove { key: Key },
}

impl<'a> Command<'a> {
    fn parse(input: &'a str) -> Result<Self, String> {
        Ok(match *input.split_whitespace().collect::<Vec<_>>().as_slice() {
            [] | ["help"] => Command::Help,
            ["insert", key, value] => Command::Insert { key: Key::parse(key)?, value },
            ["find", key] => Command::Find { key: Key::parse(key)? },
            ["remove", key] => Command::Remove { key: Key::parse(key)? },
            [command, ..] => return Err(format!("Invalid command {command:?}")),
        })
    }

    fn process(&self) -> Result<String, String> {
        match self {
            Command::Help => Ok("\
Usage: insert <key>[..<key>] <value>
Usage: find <key>[..<key>]
Usage: remove <key>[..<key>]"
                .to_string()),
            Command::Insert { key, value } => match insert(key, value.as_bytes()) {
                Ok(()) => Ok("Done".to_string()),
                Err(error) => Err(format!("{error}")),
            },
            Command::Find { key } => match find(key) {
                Ok(None) => Ok("Not found".to_string()),
                Ok(Some(value)) => match core::str::from_utf8(&value) {
                    Ok(value) => Ok(format!("Found: {value}")),
                    Err(_) => Ok(format!("Found (not UTF-8): {value:02x?}")),
                },
                Err(error) => Err(format!("{error}")),
            },
            Command::Remove { key } => match remove(key) {
                Ok(()) => Ok("Done".to_string()),
                Err(error) => Err(format!("{error}")),
            },
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
    fn parse(key: &str) -> Result<Self, String> {
        let key: Key = key.parse().map_err(|_| "Failed to parse key")?;
        let valid = match &key {
            Key::Exact(key) => *key < 4096,
            Key::Range(keys) => !keys.is_empty() && keys.end < 4096,
        };
        if !valid {
            return Err("Invalid key".to_string());
        }
        Ok(key)
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
