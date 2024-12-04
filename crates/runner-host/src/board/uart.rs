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

use std::io::ErrorKind;
use std::sync::Arc;

use tokio::net::{UnixListener, UnixStream};
use tokio::sync::mpsc::Sender;
use tokio::task::JoinHandle;
use wasefire_board_api::uart::{Api, Direction, Event};
use wasefire_board_api::{self as board, Error, Id, Support};
use wasefire_logger as log;

use crate::board::Board;
use crate::with_state;

pub enum Impl {}

pub struct Uarts([Uart; <Impl as Support<usize>>::SUPPORT]);

impl Uarts {
    pub fn new() -> Self {
        Uarts(std::array::from_fn(Uart::new))
    }

    pub fn init() {
        let uart = crate::FLAGS.dir.join("uart0");
        let _ = std::fs::remove_file(&uart);
        let listener = match UnixListener::bind(&uart) {
            Ok(x) => x,
            Err(e) => {
                log::error!("Failed to bind UART unix socket: {e}");
                return;
            }
        };
        tokio::spawn(async move {
            loop {
                match listener.accept().await {
                    Ok((stream, _)) => {
                        log::info!("UART connected.");
                        with_state(|state| {
                            let uart = state.uarts.get(Id::new(0).unwrap());
                            uart.connect(stream, &state.sender);
                        })
                    }
                    Err(e) => log::warn!("Failed to accept a UART connection: {:?}", e),
                }
            }
        });
    }

    fn get(&mut self, uart: Id<Impl>) -> &mut Uart {
        &mut self.0[*uart]
    }
}

enum Uart {
    Disconnected {
        uart: Id<Impl>,
        readable: bool,
        writable: bool,
    },
    Connected {
        uart: Id<Impl>,
        stream: Arc<UnixStream>,
        readable: Option<Listener>,
        writable: Option<Listener>,
    },
}

impl Uart {
    fn new(uart: usize) -> Self {
        let uart = Id::new(uart).unwrap();
        Uart::Disconnected { uart, readable: false, writable: false }
    }

    fn connect(&mut self, stream: UnixStream, sender: &Sender<board::Event<Board>>) {
        let uart = self.uart();
        let readable = self.has_readable();
        let writable = self.has_writable();
        *self = Uart::Connected { uart, stream: Arc::new(stream), readable: None, writable: None };
        if readable {
            self.push(sender, Direction::Read);
        }
        if writable {
            self.push(sender, Direction::Write);
        }
        self.set_readable(readable);
        self.set_writable(writable);
    }

    fn disconnect(&mut self) {
        let uart = self.uart();
        let readable = self.has_readable();
        let writable = self.has_writable();
        *self = Uart::Disconnected { uart, readable, writable }
    }

    fn uart(&self) -> Id<Impl> {
        match self {
            Uart::Disconnected { uart, .. } | Uart::Connected { uart, .. } => *uart,
        }
    }

    fn stream(&self) -> Option<&Arc<UnixStream>> {
        match self {
            Uart::Disconnected { .. } => None,
            Uart::Connected { stream, .. } => Some(stream),
        }
    }

    fn push(&self, sender: &Sender<board::Event<Board>>, direction: Direction) {
        let event = Event { uart: self.uart(), direction };
        let _ = sender.try_send(event.into());
    }

    fn set_readable(&mut self, enable: bool) {
        match self {
            Uart::Disconnected { readable, .. } => *readable = enable,
            Uart::Connected { stream, readable, .. } if enable => {
                *readable = Some(Listener::new(stream.clone(), Direction::Read))
            }
            Uart::Connected { readable, .. } => *readable = None,
        }
    }

    fn set_writable(&mut self, enable: bool) {
        match self {
            Uart::Disconnected { writable, .. } => *writable = enable,
            Uart::Connected { stream, writable, .. } if enable => {
                *writable = Some(Listener::new(stream.clone(), Direction::Write))
            }
            Uart::Connected { writable, .. } => *writable = None,
        }
    }

    fn has_readable(&self) -> bool {
        match self {
            Uart::Disconnected { readable, .. } => *readable,
            Uart::Connected { readable, .. } => readable.is_some(),
        }
    }

    fn has_writable(&self) -> bool {
        match self {
            Uart::Disconnected { writable, .. } => *writable,
            Uart::Connected { writable, .. } => writable.is_some(),
        }
    }
}

impl Support<usize> for Impl {
    const SUPPORT: usize = 1;
}

impl Api for Impl {
    fn set_baudrate(_uart: Id<Self>, _baudrate: usize) -> Result<(), Error> {
        Ok(())
    }

    fn start(_uart: Id<Self>) -> Result<(), Error> {
        Ok(())
    }

    fn stop(_uart: Id<Self>) -> Result<(), Error> {
        Ok(())
    }

    fn read(uart: Id<Self>, output: &mut [u8]) -> Result<usize, Error> {
        with_state(|state| {
            let uart = state.uarts.get(uart);
            uart.set_readable(uart.has_readable());
            match uart.stream() {
                None => Ok(0),
                Some(stream) => match stream.try_read(output) {
                    Ok(0) => {
                        uart.disconnect();
                        Ok(0)
                    }
                    x => convert(x),
                },
            }
        })
    }

    fn write(uart: Id<Self>, input: &[u8]) -> Result<usize, Error> {
        with_state(|state| {
            let uart = state.uarts.get(uart);
            uart.set_writable(uart.has_writable());
            match uart.stream() {
                None => Ok(0),
                Some(stream) => convert(stream.try_write(input)),
            }
        })
    }

    fn enable(uart: Id<Self>, direction: Direction) -> Result<(), Error> {
        with_state(|state| {
            let uart = state.uarts.get(uart);
            match direction {
                Direction::Read => uart.set_readable(true),
                Direction::Write => uart.set_writable(true),
            }
            Ok(())
        })
    }

    fn disable(uart: Id<Self>, direction: Direction) -> Result<(), Error> {
        with_state(|state| {
            let uart = state.uarts.get(uart);
            match direction {
                Direction::Read => uart.set_readable(false),
                Direction::Write => uart.set_writable(false),
            }
            Ok(())
        })
    }
}

fn convert(x: Result<usize, std::io::Error>) -> Result<usize, Error> {
    match x {
        Ok(x) => Ok(x),
        Err(e) if e.kind() == ErrorKind::WouldBlock => Ok(0),
        Err(_) => Err(Error::world(0)),
    }
}

struct Listener(JoinHandle<()>);

impl Drop for Listener {
    fn drop(&mut self) {
        self.0.abort();
    }
}

impl Listener {
    fn new(stream: Arc<UnixStream>, direction: Direction) -> Self {
        Listener(tokio::spawn(async move {
            match direction {
                Direction::Read => stream.readable().await.unwrap(),
                Direction::Write => stream.writable().await.unwrap(),
            }
            let uart = Id::new(0).unwrap();
            with_state(|state| state.uarts.get(uart).push(&state.sender, direction));
        }))
    }
}
