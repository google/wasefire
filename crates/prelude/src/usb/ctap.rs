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

//! Provides API for CTAP HID.

use sealed::sealed;
use wasefire_applet_api::usb::serial as api;

// use crate::ctap::Event;
use crate::{convert, Error};
// use crate::convert_unit;

/// Implements the [`CtapHid`](crate::ctap::CtapHid) interface for USB transport.
pub struct UsbCtapHid;

#[sealed]
impl crate::ctap::CtapHid for UsbCtapHid {
    fn read(&self, buffer: &mut [u8]) -> Result<usize, Error> {
        let params = api::read::Params { ptr: buffer.as_mut_ptr(), len: buffer.len() };
        convert(unsafe { api::read(params) })
    }

    fn write(&self, buffer: &[u8]) -> Result<usize, Error> {
        let params = api::write::Params { ptr: buffer.as_ptr(), len: buffer.len() };
        convert(unsafe { api::write(params) })
    }

    // fn flush(&self) -> Result<(), Error> {
    //     convert_unit(unsafe { api::flush() })
    // }

    //     unsafe fn register(
    //         &self, event: Event, func: extern "C" fn(*const u8), data: *const u8,
    //     ) -> Result<(), Error> {
    //         let params = api::register::Params {
    //             event: convert_event(event) as usize,
    //             handler_func: func,
    //             handler_data: data,
    //         };
    //         convert_unit(unsafe { api::register(params) })
    //     }
    //     #[doc = " Unregisters the callback for an event."]
    //     fn unregister(&self, event: Event) -> Result<(), Error> {
    //         let params = api::unregister::Params { event: convert_event(event) as usize };
    //         convert_unit(unsafe { api::unregister(params) })
    //     }
}

// fn convert_event(event: Event) -> api::Event {
//     match event {
//         Event::Read => api::Event::Read,
//         Event::Write => api::Event::Write,
//     }
// }
