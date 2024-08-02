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

//! USB serial interface.

use usb_device::class_prelude::UsbBus;
use usb_device::UsbError;
use usbd_serial::SerialPort;
use wasefire_logger as log;

use crate::Error;

/// USB serial event.
#[derive(Debug, PartialEq, Eq)]
pub enum Event {
    /// There might be data to read.
    ///
    /// This is only guaranteed to be triggered after a short read.
    Read,

    /// It might be possible to write data.
    ///
    /// This is only guaranteed to be triggered after a short write.
    Write,
}

impl<B: crate::Api> From<Event> for crate::Event<B> {
    fn from(event: Event) -> Self {
        super::Event::Serial(event).into()
    }
}

/// USB serial interface.
pub trait Api: Send {
    /// Reads from the USB serial into a buffer.
    ///
    /// Returns the number of bytes read. It could be zero if there's nothing to read.
    fn read(output: &mut [u8]) -> Result<usize, Error>;

    /// Writes from a buffer to the USB serial.
    ///
    /// Returns the number of bytes written. It could be zero if the other side is not ready.
    fn write(input: &[u8]) -> Result<usize, Error>;

    /// Flushes the USB serial.
    fn flush() -> Result<(), Error>;

    /// Enables a given event to be triggered.
    fn enable(event: &Event) -> Result<(), Error>;

    /// Disables a given event from being triggered.
    fn disable(event: &Event) -> Result<(), Error>;
}

/// Helper trait for boards using the `usbd_serial` crate.
pub trait HasSerial: Send {
    /// USB bus type.
    type UsbBus: UsbBus;

    /// Provides scoped access to the serial type.
    fn with_serial<R>(f: impl FnOnce(&mut Serial<Self::UsbBus>) -> R) -> R;
}

/// Wrapper type for boards using the `usbd_serial` crate.
pub struct WithSerial<T: HasSerial> {
    _never: !,
    _has_serial: T,
}

/// Helper struct for boards using the `usbd_serial` crate.
pub struct Serial<'a, T: UsbBus> {
    port: SerialPort<'a, T>,
    read_enabled: bool,
    write_enabled: bool,
}

impl<'a, T: UsbBus> Serial<'a, T> {
    /// Creates a new serial from a port.
    pub fn new(port: SerialPort<'a, T>) -> Self {
        Self { port, read_enabled: false, write_enabled: false }
    }

    /// Accesses the port of a serial.
    pub fn port(&mut self) -> &mut SerialPort<'a, T> {
        &mut self.port
    }

    /// Pushes events based on whether the USB serial was polled.
    pub fn tick(&mut self, polled: bool, mut push: impl FnMut(Event)) {
        if self.read_enabled && polled {
            push(Event::Read);
        }
        if self.write_enabled && self.port.dtr() {
            push(Event::Write);
        }
    }

    fn set(&mut self, event: &Event, enabled: bool) {
        match event {
            Event::Read => self.read_enabled = enabled,
            Event::Write => self.write_enabled = enabled,
        }
    }
}

impl<T: HasSerial> Api for WithSerial<T> {
    fn read(output: &mut [u8]) -> Result<usize, Error> {
        match T::with_serial(|serial| serial.port.read(output)) {
            Ok(len) => {
                log::trace!("{}{:?} = read({})", len, &output[.. len], output.len());
                Ok(len)
            }
            Err(UsbError::WouldBlock) => Ok(0),
            Err(e) => {
                log::debug!("{} = read({})", log::Debug2Format(&e), output.len());
                Err(Error::world(0))
            }
        }
    }

    fn write(input: &[u8]) -> Result<usize, Error> {
        if !T::with_serial(|serial| serial.port.dtr()) {
            // Data terminal is not ready.
            return Ok(0);
        }
        match T::with_serial(|serial| serial.port.write(input)) {
            Ok(len) => {
                log::trace!("{} = write({}{:?})", len, input.len(), input);
                Ok(len)
            }
            Err(UsbError::WouldBlock) => Ok(0),
            Err(e) => {
                log::debug!("{} = write({}{:?})", log::Debug2Format(&e), input.len(), input);
                Err(Error::world(0))
            }
        }
    }

    fn flush() -> Result<(), Error> {
        loop {
            match T::with_serial(|serial| serial.port.flush()) {
                Ok(()) => {
                    log::trace!("flush()");
                    break Ok(());
                }
                Err(UsbError::WouldBlock) => {
                    log::debug!("flush() didn't flush all data, retrying");
                    continue;
                }
                Err(e) => {
                    log::debug!("{} = flush()", log::Debug2Format(&e));
                    break Err(Error::world(0));
                }
            }
        }
    }

    fn enable(event: &Event) -> Result<(), Error> {
        T::with_serial(|serial| serial.set(event, true));
        Ok(())
    }

    fn disable(event: &Event) -> Result<(), Error> {
        T::with_serial(|serial| serial.set(event, false));
        Ok(())
    }
}
