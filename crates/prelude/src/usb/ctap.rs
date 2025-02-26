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

use alloc::boxed::Box;
use core::cell::Cell;

use wasefire_applet_api::usb::ctap as api;

pub use self::api::Event;
use crate::{Error, convert_bool, convert_unit};

/// Reads a CTAP HID packet without blocking.
///
/// Returns whether a packet was read (and thus written to the buffer).
pub fn read(packet: &mut [u8; 64]) -> Result<bool, Error> {
    let params = api::read::Params { ptr: packet.as_mut_ptr() };
    convert_bool(unsafe { api::read(params) })
}

/// Writes a CTAP HID packet without blocking.
///
/// Returns whether the packet was written (and thus read from the buffer).
pub fn write(packet: &[u8; 64]) -> Result<bool, Error> {
    let params = api::write::Params { ptr: packet.as_ptr() };
    convert_bool(unsafe { api::write(params) })
}

/// Asynchronously listens for event notifications.
pub struct Listener {
    event: api::Event,
    // Safety: This is a `Box<Cell<bool>>` we own and lend the platform as a shared borrow with a
    // lifetime starting at `api::register()` and ending at `api::unregister()`.
    notified: *const Cell<bool>,
}

impl Listener {
    /// Starts listening for the provided event until dropped.
    pub fn new(event: api::Event) -> Self {
        let notified = Box::into_raw(Box::new(Cell::new(true)));
        let handler_func = Self::call;
        let handler_data = notified as *const u8;
        let params = api::register::Params { event: event as usize, handler_func, handler_data };
        convert_unit(unsafe { api::register(params) }).unwrap();
        Listener { event, notified }
    }

    /// Returns whether the event triggered since the last call.
    pub fn is_notified(&mut self) -> bool {
        // SAFETY: `self.notified` is a `Box<Cell<bool>> we share with the platform (see field
        // invariant). We create a shared borrow for the duration of this call.
        unsafe { &*self.notified }.replace(false)
    }

    extern "C" fn call(data: *const u8) {
        // SAFETY: `data` is the shared borrow we lent the platform (see field invariant). We are
        // borrowing it back for the duration of this call.
        let notified = unsafe { &*(data as *const Cell<bool>) };
        notified.set(true);
    }
}

impl Drop for Listener {
    fn drop(&mut self) {
        let params = api::unregister::Params { event: self.event as usize };
        convert_unit(unsafe { api::unregister(params) }).unwrap();
        // SAFETY: `self.notified` is a `Box<Cell<bool>>` we own back, now that the lifetime of the
        // platform borrow is over (see field invariant).
        drop(unsafe { Box::from_raw(self.notified as *mut Cell<bool>) });
    }
}
