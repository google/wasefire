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

use std::io::{Read, Write};
use std::path::PathBuf;
use std::time::Instant;

use anyhow::{Result, anyhow};
use clap::Parser;
use common::{Deserialize, Deserializer, Error, Request, Response, Serialize, Serializer};
use wasefire_cli_tools::action::usb_serial::ConnectionOptions;
use wasefire_cli_tools::action::usb_serial::serialport::SerialPort;

#[derive(Parser)]
struct Flags {
    #[command(flatten)]
    options: ConnectionOptions,
    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, clap::Subcommand)]
enum Command {
    /// Generates a key.
    GenerateKey {
        /// The key to generate.
        key: usize,
    },

    /// Deletes a key.
    DeleteKey {
        /// The key to delete.
        key: usize,
    },

    /// Encrypts a message given a key.
    Encrypt {
        /// The key to encrypt.
        key: usize,

        #[clap(flatten)]
        input: Input,

        #[clap(flatten)]
        output: Output,
    },

    /// Decrypts a message given a key.
    Decrypt {
        /// The key to decrypt.
        key: usize,

        #[clap(flatten)]
        input: Input,

        #[clap(flatten)]
        output: Output,
    },

    /// Imports a key.
    ImportKey {
        /// The key to import.
        key: usize,

        #[clap(flatten)]
        input: Input,
    },

    /// Exports a key.
    ExportKey {
        /// The key to export.
        key: usize,

        #[clap(flatten)]
        output: Output,
    },
}

#[derive(Debug, clap::Args)]
struct Input {
    /// The input file (defaults to stdin).
    #[arg(long)]
    input: Option<PathBuf>,
}

#[derive(Debug, clap::Args)]
struct Output {
    /// The output file (defaults to stdout).
    #[arg(long)]
    output: Option<PathBuf>,
}

impl Command {
    fn request(&self) -> Result<Request> {
        Ok(match self {
            Command::GenerateKey { key } => Request::GenerateKey { key: *key },
            Command::DeleteKey { key } => Request::DeleteKey { key: *key },
            Command::Encrypt { key, input, .. } => {
                Request::Encrypt { key: *key, nonce: [0; 8], data: input.read()? }
            }
            Command::Decrypt { key, input, .. } => {
                Request::Decrypt { key: *key, nonce: [0; 8], data: input.read()? }
            }
            Command::ImportKey { key, input } => {
                let secret = input.read()?.try_into().map_err(|_| anyhow!("wrong key size"))?;
                Request::ImportKey { key: *key, secret }
            }
            Command::ExportKey { key, .. } => Request::ExportKey { key: *key },
        })
    }

    fn response(&self, response: &Response) -> Result<()> {
        match (self, response) {
            (Command::GenerateKey { key }, Response::GenerateKey) => {
                println!("Generated key {key}.");
            }
            (Command::DeleteKey { key }, Response::DeleteKey) => {
                println!("Deleted key {key}.");
            }
            (Command::Encrypt { output, .. }, Response::Encrypt { data }) => {
                output.write(data)?;
            }
            (Command::Decrypt { output, .. }, Response::Decrypt { data }) => {
                output.write(data)?;
            }
            (Command::ImportKey { key, .. }, Response::ImportKey) => {
                println!("Imported key {key}.");
            }
            (Command::ExportKey { output, .. }, Response::ExportKey { secret }) => {
                output.write(secret)?;
            }
            x => panic!("{x:?}"),
        }
        Ok(())
    }
}

impl Input {
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

impl Output {
    fn write(&self, data: &[u8]) -> Result<()> {
        match &self.output {
            None => std::io::stdout().write_all(data)?,
            Some(file) => std::fs::write(file, data)?,
        }
        Ok(())
    }
}

struct Serial(Box<dyn SerialPort>);

impl Serializer for Serial {
    fn write_all(&mut self, data: &[u8]) -> Result<(), Error> {
        log::info!("Write {data:02x?}");
        self.0.write_all(data).map_err(|_| Error::UsbError)
    }

    fn flush(&mut self) -> Result<(), Error> {
        log::info!("Flush");
        self.0.flush().map_err(|_| Error::UsbError)
    }
}

impl Deserializer for Serial {
    fn read_exact(&mut self, data: &mut [u8]) -> Result<(), Error> {
        let result = self.0.read_exact(data).map_err(|_| Error::UsbError);
        log::info!("Read {data:02x?}");
        result
    }
}

fn main() -> Result<()> {
    env_logger::init();
    let flags = Flags::parse();
    let mut serial = Serial(flags.options.connect()?);
    log::info!("Connected to serial port.");
    let start = Instant::now();
    flags.command.request()?.serialize(&mut serial)?;
    let response = <Result<Response, Error>>::deserialize(&mut serial)??;
    let finish = Instant::now();
    eprintln!("Executed in {:?}", finish - start);
    flags.command.response(&response)?;
    Ok(())
}
