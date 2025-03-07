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

use std::io::Read;
use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;
use serialport::SerialPort;
use wasefire_cli_tools::action::usb_serial::ConnectionOptions;

/// Updates a firmware through USB serial.
#[derive(Parser)]
struct Flags {
    #[command(flatten)]
    options: ConnectionOptions,

    /// The firmware file in binary format (defaults to stdin).
    input: Option<PathBuf>,
}

impl Flags {
    fn read(&self) -> Result<Vec<u8>> {
        Ok(match &self.input {
            None => {
                let mut result = Vec::new();
                std::io::stdin().read_to_end(&mut result)?;
                result
            }
            Some(file) => std::fs::read(file)?,
        })
    }
}

struct Serial(Box<dyn SerialPort>);

impl Serial {
    fn write(&mut self, data: &[u8]) -> Result<()> {
        Ok(self.0.write_all(data)?)
    }
}

fn main() -> Result<()> {
    env_logger::init();
    let flags = Flags::parse();
    let mut serial = Serial(flags.options.connect()?);
    log::info!("Connected to serial port.");
    let firmware = flags.read()?;
    let length = firmware.len() as u32;
    log::info!("Sending {length} bytes of firmware.");
    serial.write(&length.to_be_bytes())?;
    serial.write(&firmware)?;
    Ok(())
}
