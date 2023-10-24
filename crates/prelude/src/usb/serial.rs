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

//! Provides API for USB serial.

use wasefire_applet_api::usb::serial as api;

use crate::serial::{Event, Serial};
use crate::usb::{convert, Error};

/// Implements the [`Serial`] interface for the USB serial.
pub struct UsbSerial;

impl Serial for UsbSerial {
    type Error = Error;

    fn read(&self, buffer: &mut [u8]) -> Result<usize, Error> {
        let params = api::read::Params { ptr: buffer.as_mut_ptr(), len: buffer.len() };
        let api::read::Results { len } = unsafe { api::read(params) };
        convert(len)
    }

    fn write(&self, buffer: &[u8]) -> Result<usize, Error> {
        let params = api::write::Params { ptr: buffer.as_ptr(), len: buffer.len() };
        let api::write::Results { len } = unsafe { api::write(params) };
        convert(len)
    }

    fn flush(&self) -> Result<(), Self::Error> {
        let api::flush::Results { res } = unsafe { api::flush() };
        convert(res).map(|_| ())
    }

    unsafe fn register(
        &self, event: Event, func: extern "C" fn(*const u8), data: *const u8,
    ) -> Result<(), Error> {
        let params = api::register::Params {
            event: convert_event(event) as usize,
            handler_func: func,
            handler_data: data,
        };
        unsafe { api::register(params) };
        Ok(())
    }

    fn unregister(&self, event: Event) -> Result<(), Error> {
        let params = api::unregister::Params { event: convert_event(event) as usize };
        unsafe { api::unregister(params) };
        Ok(())
    }
}

fn convert_event(event: Event) -> api::Event {
    match event {
        Event::Read => api::Event::Read,
        Event::Write => api::Event::Write,
    }
}
