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

use anyhow::{Result, bail};
pub use serialport;
use serialport::{SerialPort, SerialPortType};

/// Options to connect to a USB serial.
#[derive(Clone, clap::Args)]
pub struct ConnectionOptions {
    /// Filter the USB serial to connect using the platform serial number.
    #[arg(long)]
    serial: Option<String>,

    /// Timeout to send or receive with the USB serial.
    #[arg(long, default_value = "1s")]
    timeout: humantime::Duration,
}

impl ConnectionOptions {
    /// Establishes a connection.
    pub fn connect(&self) -> Result<Box<dyn SerialPort>> {
        let ConnectionOptions { serial, timeout } = self;
        for info in serialport::available_ports()? {
            let path = info.port_name;
            let SerialPortType::UsbPort(info) = info.port_type else { continue };
            if info.vid != 0x18d1 || info.pid != 0x0239 {
                continue;
            }
            match (serial, &info.serial_number) {
                (None, _) => (),
                (Some(expected), Some(actual)) if actual == expected => (),
                _ => continue,
            }
            return Ok(serialport::new(path, 19200).timeout(**timeout).open()?);
        }
        bail!("no available port");
    }
}
