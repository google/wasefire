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

use std::fmt::{Display, Formatter};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Duration;

use anyhow::{bail, ensure, Result};
use data_encoding::HEXLOWER_PERMISSIVE as HEX;
use rusb::{Device, GlobalContext};
use wasefire_protocol::{self as service, Connection, ConnectionExt};

type PlatformInfo = wasefire_wire::Yoke<service::platform::Info<'static>>;

#[derive(Clone)]
pub(crate) enum Protocol {
    Usb(ProtocolUsb),
    Unix(PathBuf),
    Tcp(SocketAddr),
}

#[derive(Clone)]
pub(crate) enum ProtocolUsb {
    Auto,
    Serial(Hex),
    BusDev { bus: u8, dev: u8 },
}

#[derive(Clone)]
pub(crate) struct Hex(pub(crate) Vec<u8>);

impl Protocol {
    pub(crate) async fn connect(&self, timeout: Duration) -> Result<Box<dyn Connection>> {
        match self {
            Protocol::Usb(x) => x.connect(timeout).await,
            Protocol::Unix(x) => {
                Ok(Box::new(wasefire_protocol_tokio::Connection::new_unix(x).await?))
            }
            Protocol::Tcp(x) => {
                Ok(Box::new(wasefire_protocol_tokio::Connection::new_tcp(*x).await?))
            }
        }
    }
}

impl ProtocolUsb {
    async fn connect(&self, timeout: Duration) -> Result<Box<dyn Connection>> {
        let context = GlobalContext::default();
        let mut matches = Vec::new();
        for candidate in wasefire_protocol_usb::list(&context)? {
            let mut connection = candidate.connect(timeout)?;
            let info = connection.call::<service::PlatformInfo>(()).await?;
            if !self.matches(connection.device(), &info) {
                continue;
            }
            matches.push((connection, info));
        }
        match matches.len() {
            1 => Ok(Box::new(matches.pop().unwrap().0)),
            0 => bail!("no connected platforms matching {self}"),
            _ => {
                eprintln!("Multiple connected platforms matching {self}:");
                for (connection, info) in matches {
                    let serial = ProtocolUsb::Serial(Hex(info.get().serial.to_vec()));
                    let bus = connection.device().bus_number();
                    let dev = connection.device().address();
                    let busdev = ProtocolUsb::BusDev { bus, dev };
                    eprintln!("- {serial} or {busdev}");
                }
                bail!("more than one connected platform matching {self}");
            }
        }
    }

    fn matches(&self, device: &Device<GlobalContext>, info: &PlatformInfo) -> bool {
        match self {
            ProtocolUsb::Auto => true,
            ProtocolUsb::Serial(x) => x.0 == info.get().serial,
            ProtocolUsb::BusDev { bus, dev } => {
                *bus == device.bus_number() && *dev == device.address()
            }
        }
    }
}

impl Display for Protocol {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Protocol::Usb(x) => write!(f, "usb:{x}"),
            Protocol::Unix(x) => write!(f, "unix:{}", x.display()),
            Protocol::Tcp(x) => write!(f, "tcp:{x}"),
        }
    }
}

impl FromStr for Protocol {
    type Err = anyhow::Error;
    fn from_str(input: &str) -> anyhow::Result<Self> {
        let input: Vec<_> = input.split(':').collect();
        ensure!(!input.is_empty(), "protocol cannot be empty");
        match input[0] {
            "usb" => match input.len() {
                1 => Ok(Protocol::Usb(ProtocolUsb::Auto)),
                2 => Ok(Protocol::Usb(ProtocolUsb::Serial(input[1].parse()?))),
                3 => {
                    let bus = input[1].parse()?;
                    let dev = input[2].parse()?;
                    Ok(Protocol::Usb(ProtocolUsb::BusDev { bus, dev }))
                }
                _ => bail!("invalid number of arguments"),
            },
            "unix" => {
                let path = match input.len() {
                    1 => "/tmp/wasefire",
                    2 => input[1],
                    _ => bail!("invalid number of arguments"),
                };
                Ok(Protocol::Unix(path.into()))
            }
            "tcp" => {
                let addr = match input.len() {
                    1 => "127.0.0.1:3457".parse()?,
                    3 => SocketAddr::new(input[1].parse()?, input[2].parse()?),
                    _ => bail!("invalid number of arguments"),
                };
                Ok(Protocol::Tcp(addr))
            }
            x => bail!("invalid protocol {x:?}"),
        }
    }
}

impl Display for ProtocolUsb {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ProtocolUsb::Auto => write!(f, "usb"),
            ProtocolUsb::Serial(x) => write!(f, "usb:{x}"),
            ProtocolUsb::BusDev { bus, dev } => write!(f, "usb:{bus:03}:{dev:03}"),
        }
    }
}

impl Display for Hex {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        HEX.encode(&self.0).fmt(f)
    }
}

impl FromStr for Hex {
    type Err = data_encoding::DecodeError;
    fn from_str(input: &str) -> std::result::Result<Self, Self::Err> {
        Ok(Hex(HEX.decode(input.as_bytes())?))
    }
}
