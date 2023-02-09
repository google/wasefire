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

//! Provides API for clocks and timers.
//!
//! For now, only timers are provided.

use alloc::boxed::Box;
use alloc::rc::Rc;
use core::cell::Cell;
use core::time::Duration;

use api::clock as api;

pub use self::api::Mode;
pub use self::api::Mode::*;

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
    handler: *mut H,
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
        let handler_data = handler as *mut u8;
        let params = api::allocate::Params { handler_func, handler_data };
        let api::allocate::Results { id } = unsafe { api::allocate(params) };
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
        unsafe { api::start(params) };
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
        unsafe { api::stop(api::stop::Params { id: self.id }) };
    }

    /// Drops the timer but keeps it running.
    ///
    /// This is really only useful for periodic timers. Otherwise it's leaking the handler.
    pub fn leak(self) {
        core::mem::forget(self);
    }

    extern "C" fn call(data: *mut u8) {
        let handler = unsafe { &mut *(data as *mut H) };
        handler.event();
    }
}

impl<H: Handler> Drop for Timer<H> {
    fn drop(&mut self) {
        if self.running.get() {
            self.stop();
        }
        let params = api::free::Params { id: self.id };
        unsafe { api::free(params) };
        unsafe { Box::from_raw(self.handler) };
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
