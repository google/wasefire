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

//! Provides API for button or touch sensors.
//!
//! Button or touch sensors are abstracted with:
//! - They have 2 states: `Pressed` or `Released`.
//! - They can trigger a callback on any state change.

use alloc::boxed::Box;

use wasefire_applet_api::button as api;

pub use self::api::State;
pub use self::api::State::*;
use crate::{convert, convert_unit};

/// Returns the number of available buttons on the board.
pub fn count() -> usize {
    convert(unsafe { api::count() }).unwrap_or(0)
}

/// Provides callback support for button events.
pub trait Handler: 'static {
    /// Called when a button changed state.
    ///
    /// The `state` argument is the new state of the button.
    fn event(&self, state: State);
}

impl<F: Fn(State) + 'static> Handler for F {
    fn event(&self, state: State) {
        self(state)
    }
}

/// Provides listening support for button events.
#[must_use]
pub struct Listener<H: Handler> {
    button: usize,
    handler: *const H,
}

impl<H: Handler> Listener<H> {
    /// Starts listening for button events.
    ///
    /// The `button` argument is the index of the button to listen events for. It must be less than
    /// [count()]. The `handler` argument is the callback to be called on events. Note that it may
    /// be an `Fn(state: State)` closure, see [Handler::event()] for callback documentation.
    ///
    /// The listener stops listening when dropped.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// Listener::new(index, |state| debug!("Button has been {state:?}"))
    /// ```
    pub fn new(button: usize, handler: H) -> Self {
        let handler_func = Self::call;
        let handler = Box::into_raw(Box::new(handler));
        let handler_data = handler as *const u8;
        let params = api::register::Params { button, handler_func, handler_data };
        convert_unit(unsafe { api::register(params) }).unwrap();
        Listener { button, handler }
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

    extern "C" fn call(data: *const u8, state: usize) {
        let handler = unsafe { &*(data as *const H) };
        let state = state.into();
        handler.event(state);
    }
}

impl<H: Handler> Drop for Listener<H> {
    fn drop(&mut self) {
        let params = api::unregister::Params { button: self.button };
        convert_unit(unsafe { api::unregister(params) }).unwrap();
        drop(unsafe { Box::from_raw(self.handler as *mut H) });
    }
}
