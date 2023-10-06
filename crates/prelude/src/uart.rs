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

//! Provides API for UART.

use wasefire_applet_api::uart as api;

use crate::serial::{Event, Serial};

/// UART error.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Error;

/// Returns the number of available UARTs on the board.
pub fn count() -> usize {
    let api::count::Results { cnt } = unsafe { api::count() };
    cnt
}

/// Reads from a UART into a buffer without blocking.
///
/// Returns how many bytes were read (and thus written to the buffer). This function does not block,
/// so if there are no data available for read, zero is returned.
pub fn read(uart: usize, buf: &mut [u8]) -> Result<usize, Error> {
    let params = api::read::Params { uart, ptr: buf.as_mut_ptr(), len: buf.len() };
    let api::read::Results { len } = unsafe { api::read(params) };
    convert(len)
}

/// Writes from a buffer to a UART.
///
/// Returns how many bytes were written (and thus read from the buffer). This function does not
/// block, so if the serial is not ready for write, zero is returned.
pub fn write(uart: usize, buf: &[u8]) -> Result<usize, Error> {
    let params = api::write::Params { uart, ptr: buf.as_ptr(), len: buf.len() };
    let api::write::Results { len } = unsafe { api::write(params) };
    convert(len)
}

/// Implements the [`Serial`] interface for UART.
pub struct Uart(pub usize);

impl Serial for Uart {
    type Error = Error;

    fn read(&self, buffer: &mut [u8]) -> Result<usize, Error> {
        read(self.0, buffer)
    }

    fn write(&self, buffer: &[u8]) -> Result<usize, Error> {
        write(self.0, buffer)
    }

    fn flush(&self) -> Result<(), Error> {
        Ok(())
    }

    fn register(
        &self, event: Event, func: extern "C" fn(*const u8), data: *const u8,
    ) -> Result<(), Error> {
        let params = api::register::Params {
            uart: self.0,
            event: convert_event(event) as usize,
            handler_func: func,
            handler_data: data,
        };
        unsafe { api::register(params) };
        Ok(())
    }

    fn unregister(&self, event: Event) -> Result<(), Error> {
        let params = api::unregister::Params { uart: self.0, event: convert_event(event) as usize };
        unsafe { api::unregister(params) };
        Ok(())
    }
}

fn convert(code: isize) -> Result<usize, Error> {
    if code < 0 {
        Err(Error)
    } else {
        Ok(code as usize)
    }
}

fn convert_event(event: Event) -> api::Event {
    match event {
        Event::Read => api::Event::Read,
        Event::Write => api::Event::Write,
    }
}
