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

use alloc::boxed::Box;

use bytemuck::Zeroable;
use wasefire_applet_api::radio::ble as api;
pub use wasefire_applet_api::radio::ble::{Advertisement, Event};

use crate::{convert_bool, Error};

/// Reads the next advertisement packet, if any.
pub fn read_advertisement() -> Result<Option<Advertisement>, Error> {
    let mut result = Advertisement::zeroed();
    let params = api::read_advertisement::Params { ptr: &mut result as *mut _ as *mut u8 };
    let api::read_advertisement::Results { res } = unsafe { api::read_advertisement(params) };
    Ok(convert_bool(res)?.then_some(result))
}

/// Provides callback support for BLE events.
pub trait Handler: 'static {
    /// Called when a BLE event triggers.
    fn event(&self, event: Event);
}

impl<F: Fn(Event) + 'static> Handler for F {
    fn event(&self, event: Event) {
        self(event)
    }
}

/// Provides listening support for BLE events.
#[must_use]
pub struct Listener<H: Handler> {
    handler: *mut H,
}

impl<H: Handler> Listener<H> {
    /// Starts listening for BLE events.
    ///
    /// The listener stops listening when dropped.
    pub fn new(event: Event, handler: H) -> Self {
        assert!(matches!(event, Event::Advertisement), "only advertisement events are supported");
        let event = event as u32;
        let handler_func = Self::call;
        let handler = Box::into_raw(Box::new(handler));
        let handler_data = handler as *const u8;
        unsafe { api::register(api::register::Params { event, handler_func, handler_data }) };
        Listener { handler }
    }

    /// Stops listening.
    ///
    /// This is equivalent to calling `core::mem::drop()`.
    pub fn stop(self) {
        core::mem::drop(self);
    }

    /// Drops the listener but continues listening.
    ///
    /// This is equivalent to calling `core::mem::forget()`. This can be useful if the listener is
    /// created deeply in the stack but the callback must continue processing events until the
    /// applet exits or traps.
    pub fn leak(self) {
        core::mem::forget(self);
    }

    extern "C" fn call(data: *const u8) {
        let handler = unsafe { &*(data as *const H) };
        handler.event(Event::Advertisement);
    }
}

impl<H: Handler> Drop for Listener<H> {
    fn drop(&mut self) {
        let params = api::unregister::Params { event: Event::Advertisement as u32 };
        unsafe { api::unregister(params) };
        drop(unsafe { Box::from_raw(self.handler) });
    }
}
