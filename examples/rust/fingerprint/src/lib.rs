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

//! Demonstrates simple fingerprint usage.
//!
//! The applet uses the platform protocol applet RPC system for its interface. It is a simple
//! text-based interface and describes its usage when receiving "help" as a request:
//!
//!     ( echo help; cat ) | wasefire applet-rpc --repl

#![no_std]
#![feature(try_blocks)]
wasefire::applet!();

use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::str::FromStr;
use core::time::Duration;

use data_encoding::HEXLOWER_PERMISSIVE as HEX;
use wasefire::error::Code;
use wasefire::fingerprint::matcher::{
    self, Enroll, EnrollProgress, Identify, IdentifyResult, delete_template, list_templates,
};
use wasefire::fingerprint::sensor::{self, Capture, Image};
use wasefire::timer::Timeout;

fn main() {
    rpc::Listener::new(&platform::protocol::RpcProtocol, handler).leak();
}

fn handler(request: Vec<u8>) -> Vec<u8> {
    // Parse and process the request.
    let result: Result<String, String> = try {
        let request = String::from_utf8(request).map_err(|_| "Request is not UTF-8")?;
        Command::parse(&request)?.process()?
    };
    // Format output including error and next prompt.
    let mut output = result.unwrap_or_else(|error| format!("Error: {error}"));
    output.push_str("\n> ");
    output.into_bytes()
}

enum Command {
    Help,
    Capture,
    Enroll,
    Identify { id: Option<TemplateId> },
    Delete { id: Option<TemplateId> },
    List,
}

impl Command {
    fn parse(input: &str) -> Result<Self, String> {
        Ok(match *input.split_whitespace().collect::<Vec<_>>().as_slice() {
            [] | ["help"] => Command::Help,
            ["capture"] => Command::Capture,
            ["enroll"] => Command::Enroll,
            ["identify"] => Command::Identify { id: None },
            ["identify", id] => Command::Identify { id: Some(TemplateId::parse(id)?) },
            ["delete"] => Command::Delete { id: None },
            ["delete", id] => Command::Delete { id: Some(TemplateId::parse(id)?) },
            ["list"] => Command::List,
            [command, ..] => return Err(format!("Invalid command {command:?}")),
        })
    }

    fn process(&self) -> Result<String, String> {
        match self {
            Command::Help => Ok("\
Usage: capture
Usage: enroll
Usage: identify [<template_id>]
Usage: delete [<template_id>]
Usage: list"
                .to_string()),
            Command::Capture if sensor::is_supported() => match capture() {
                Ok(image) => {
                    use core::fmt::Write as _;
                    let width = image.width;
                    let height = image.data.len() / image.width;
                    let w = core::cmp::min(16, width);
                    let h = core::cmp::min(8, height);
                    let mut output = String::new();
                    for i in 0 .. h {
                        writeln!(output, "{:02x?}", &image.data[i * width ..][.. w]).unwrap();
                    }
                    write!(output, "Size: {width}x{height} ({w}x{h} displayed)").unwrap();
                    Ok(output)
                }
                Err(error) => Err(format!("{error}")),
            },
            Command::Enroll if matcher::is_supported() => match enroll() {
                Ok(id) => Ok(format!("Template ID: {id}")),
                Err(error) => Err(format!("{error}")),
            },
            Command::Identify { id } if matcher::is_supported() => match identify(id) {
                Ok(None) => Ok("No match".to_string()),
                Ok(Some(id)) => Ok(format!("Matched {id}")),
                Err(error) => Err(format!("{error}")),
            },
            Command::Delete { id } if matcher::is_supported() => match delete(id) {
                Ok(()) => Ok("Done".to_string()),
                Err(error) => Err(format!("{error}")),
            },
            Command::List if matcher::is_supported() => match list_templates() {
                Ok(templates) => {
                    use core::fmt::Write as _;
                    let mut list = String::new();
                    write!(list, "There are {} enrolled fingers.", templates.len()).unwrap();
                    for template in templates.iter() {
                        write!(list, "\n- {}", HEX.encode(template)).unwrap();
                    }
                    Ok(list)
                }
                Err(error) => Err(format!("{error}")),
            },
            _ => Err("Unsupported command".to_string()),
        }
    }
}

struct TemplateId(Vec<u8>);

impl FromStr for TemplateId {
    type Err = data_encoding::DecodeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        HEX.decode(s.as_bytes()).map(TemplateId)
    }
}

impl core::fmt::Display for TemplateId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(&HEX.encode(&self.0))
    }
}

impl TemplateId {
    fn parse(id: &str) -> Result<Self, String> {
        id.parse().map_err(|_| "Failed to parse template id".to_string())
    }
}

fn capture() -> Result<Image, Error> {
    let capture = Capture::new()?;
    let timeout = Timeout::new(Duration::from_secs(10));
    scheduling::wait_until(|| capture.is_done() || timeout.is_over());
    if timeout.is_over() {
        debug!("Timed out.");
        capture.abort()?;
        Err(Error::user(Code::NotEnough))
    } else {
        capture.result()
    }
}

fn enroll() -> Result<TemplateId, Error> {
    let enroll = Enroll::new()?;
    let timeout = Timeout::new(Duration::from_secs(60));
    scheduling::wait_until(|| match enroll.progress() {
        None => true,
        Some(_) if timeout.is_over() => true,
        Some(EnrollProgress { detected, remaining }) => {
            let percent = detected * 100 / (detected + remaining.unwrap_or(1));
            debug!("Enrollment progress: {percent}%.");
            false
        }
    });
    if timeout.is_over() {
        debug!("Timed out.");
        enroll.abort()?;
        Err(Error::user(Code::NotEnough))
    } else {
        let id = enroll.result()?;
        Ok(TemplateId(id.into_vec()))
    }
}

fn identify(id: &Option<TemplateId>) -> Result<Option<TemplateId>, Error> {
    let identify = Identify::new(id.as_ref().map(|x| x.0.as_slice()))?;
    let timeout = Timeout::new(Duration::from_secs(10));
    scheduling::wait_until(|| identify.is_done() || timeout.is_over());
    if timeout.is_over() {
        debug!("Timed out.");
        identify.abort()?;
        Err(Error::user(Code::NotEnough))
    } else {
        Ok(match identify.result()? {
            IdentifyResult::NoMatch => None,
            IdentifyResult::Match { template } => Some(TemplateId(template.into_vec())),
        })
    }
}

fn delete(id: &Option<TemplateId>) -> Result<(), Error> {
    delete_template(id.as_ref().map(|x| x.0.as_slice()))
}
