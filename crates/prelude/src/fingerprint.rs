// Copyright 2025 Google LLC
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

//! Provides fingerprint matcher and/or sensor.

use alloc::boxed::Box;

use wasefire_applet_api::fingerprint as api;
use wasefire_common::ptr::SharedPtr;
use wasefire_error::Error;

use crate::convert_unit;

#[cfg(feature = "api-fingerprint-matcher")]
pub mod matcher;
#[cfg(feature = "api-fingerprint-sensor")]
pub mod sensor;

/// Provides callback support for finger detection.
pub trait Handler: 'static {
    /// Called when a finger is detected.
    fn event(&self);
}

impl<F: Fn() + 'static> Handler for F {
    fn event(&self) {
        self()
    }
}

/// Provides listening support for finger detection.
#[must_use]
pub struct Listener<H: Handler> {
    // Safety: This is a `Box<H>` we own and lend the platform as a shared borrow `&'a H` where
    // `'a` starts at `api::register()` and ends at `api::unregister()`.
    handler: SharedPtr<H>,
}

impl<H: Handler> Listener<H> {
    /// Starts listening for finger detection.
    ///
    /// The `handler` argument is the callback to be called each time a finger is detected. Note
    /// that it may be an `Fn()` closure.
    ///
    /// The listener stops listening when dropped.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// Listener::new(|| debug!("Fingerprint was touched"))?
    /// ```
    pub fn new(handler: H) -> Result<Self, Error> {
        let handler_func = Self::call;
        let handler = Box::into_raw(Box::new(handler));
        let handler_data = handler as *const u8;
        let params = api::register::Params { handler_func, handler_data };
        convert_unit(unsafe { api::register(params) })?;
        Ok(Listener { handler: SharedPtr(handler) })
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
        // SAFETY: `data` is the `&H` we lent the platform (see `Listener::handler`). We are
        // borrowing it back for the duration of this call.
        let handler = unsafe { &*(data as *const H) };
        handler.event();
    }
}

impl<H: Handler> Drop for Listener<H> {
    fn drop(&mut self) {
        convert_unit(unsafe { api::unregister() }).unwrap();
        // SAFETY: `self.handler` is a `Box<H>` we own back, now that the lifetime of the platform
        // borrow is over (see `Listener::handler`).
        drop(unsafe { Box::from_raw(self.handler.0 as *mut H) });
    }
}
