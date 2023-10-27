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

use sealed::sealed;
use wasefire_applet_api::uart as api;

use crate::serial::Event;

/// UART error.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Error;

/// Returns the number of available UARTs on the board.
pub fn count() -> usize {
    let api::count::Results { cnt } = unsafe { api::count() };
    cnt
}

/// Implements the [`Serial`] interface for UART.
pub struct Uart(pub usize);

#[sealed]
impl crate::serial::Serial for Uart {
    type Error = Error;

    fn read(&self, buffer: &mut [u8]) -> Result<usize, Error> {
        let params =
            api::read::Params { uart: self.0, ptr: buffer.as_mut_ptr(), len: buffer.len() };
        let api::read::Results { len } = unsafe { api::read(params) };
        convert(len)
    }

    fn write(&self, buffer: &[u8]) -> Result<usize, Error> {
        let params = api::write::Params { uart: self.0, ptr: buffer.as_ptr(), len: buffer.len() };
        let api::write::Results { len } = unsafe { api::write(params) };
        convert(len)
    }

    fn flush(&self) -> Result<(), Error> {
        Ok(())
    }

    unsafe fn register(
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
