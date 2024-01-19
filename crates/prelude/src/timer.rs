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

//! Provides API for timers.

use alloc::boxed::Box;
use alloc::rc::Rc;
use core::cell::Cell;
use core::time::Duration;

use wasefire_applet_api::timer as api;

pub use self::api::Mode;
pub use self::api::Mode::*;
use crate::{convert, convert_unit};

/// Provides callback support for timer events.
pub trait Handler: 'static {
    /// Called when a timer fires.
    fn event(&self);
}

impl<F: Fn() + 'static> Handler for F {
    fn event(&self) {
        self()
    }
}

/// Interface to a timer in the board.
#[must_use]
pub struct Timer<H: Handler> {
    id: usize,
    running: Cell<bool>,
    handler: *const H,
}

impl<H: Handler> Timer<H> {
    /// Allocates a timer in the board.
    ///
    /// If all timers are allocated, this will trap. This can be improved in the future to return an
    /// error (like `EBUSY`).
    ///
    /// Note that the `handler` may be a simpl `Fn()` closure.
    pub fn new(handler: H) -> Self {
        let handler_func = Self::call;
        let handler = Box::into_raw(Box::new(handler));
        let handler_data = handler as *const u8;
        let params = api::allocate::Params { handler_func, handler_data };
        let id = convert(unsafe { api::allocate(params) }).unwrap();
        Timer { id, running: Cell::new(false), handler }
    }

    /// Starts the timer.
    ///
    /// This is a lower-level version that directly takes the number of milliseconds.
    pub fn start_ms(&self, mode: Mode, duration_ms: usize) {
        if self.running.replace(true) {
            return;
        }
        let params = api::start::Params { id: self.id, mode: mode as usize, duration_ms };
        convert_unit(unsafe { api::start(params) }).unwrap();
    }

    /// Starts the timer.
    ///
    /// The `periodic` argument specifies whether the timer should periodically fire or if it should
    /// fire only once. The `duration_ms` is the time (resp. period) in milliseconds until (resp. at
    /// which) the timer fires.
    pub fn start(&self, mode: Mode, duration: Duration) {
        self.start_ms(mode, duration.as_millis() as usize)
    }

    /// Stops the timer, if still running.
    ///
    /// Note that due to concurrency, it is possible that the timer already fired but the callback
    /// didn't get a change to execute. In this case, the callback will still execute when it gets a
    /// chance. This function just tells the platform to not schedule the callback from now on.
    pub fn stop(&self) {
        if !self.running.replace(false) {
            return;
        }
        convert_unit(unsafe { api::stop(api::stop::Params { id: self.id }) }).unwrap();
    }

    /// Drops the timer but keeps it running.
    ///
    /// This is really only useful for periodic timers. Otherwise it's leaking the handler.
    pub fn leak(self) {
        core::mem::forget(self);
    }

    extern "C" fn call(data: *const u8) {
        let handler = unsafe { &*(data as *const H) };
        handler.event();
    }
}

impl<H: Handler> Drop for Timer<H> {
    fn drop(&mut self) {
        if self.running.get() {
            self.stop();
        }
        let params = api::free::Params { id: self.id };
        convert_unit(unsafe { api::free(params) }).unwrap();
        drop(unsafe { Box::from_raw(self.handler as *mut H) });
    }
}

/// Sleeps for a given duration in milliseconds.
///
/// This is a convenience function to avoid creating a timer and a callback.
pub fn sleep_ms(duration_ms: usize) {
    let done = Rc::new(Cell::new(false));
    let timer = Timer::new({
        let done = done.clone();
        move || done.set(true)
    });
    timer.start_ms(Oneshot, duration_ms);
    while !done.get() {
        crate::scheduling::wait_for_callback();
    }
}

/// Sleeps for a given duration.
///
/// The time resolution is milli-seconds (rounded down).
pub fn sleep(duration: Duration) {
    sleep_ms(duration.as_millis() as usize)
}
