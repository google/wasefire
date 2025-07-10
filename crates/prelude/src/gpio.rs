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

//! Provides low-level API for GPIOs.
//!
//! See [`crate::button`] and [`crate::led`] for higher-level API for GPIOs.

use alloc::boxed::Box;

use wasefire_applet_api::gpio as api;
pub use wasefire_applet_api::gpio::{Event, InputConfig, OutputConfig};
use wasefire_error::Error;

use crate::{convert, convert_bool, convert_unit};

/// Returns the number of available GPIOs on the board.
pub fn count() -> usize {
    convert(unsafe { api::count() }).unwrap_or(0)
}

/// GPIO configuration.
#[derive(Debug)]
pub struct Config {
    /// Input configuration.
    pub input: InputConfig,

    /// Output configuration.
    pub output: OutputConfig,

    /// Initial output value.
    pub initial: bool,
}

impl Config {
    /// Disconnects the GPIO.
    pub fn disconnected() -> Self {
        Config::input(InputConfig::Disabled)
    }

    /// Configures the GPIO for input.
    ///
    /// If you don't have specific needs, then `Floating` is most common configuration.
    pub fn input(input: InputConfig) -> Self {
        Config::new(input, OutputConfig::Disabled, false)
    }

    /// Configures the GPIO for output.
    ///
    /// If you don't have specific needs, then `PushPull` is most common configuration.
    pub fn output(output: OutputConfig, initial: bool) -> Self {
        Config::new(InputConfig::Disabled, output, initial)
    }

    /// Creates a new configuration (possibly for both input and output).
    pub fn new(input: InputConfig, output: OutputConfig, initial: bool) -> Self {
        Config { input, output, initial }
    }

    fn mode(&self) -> usize {
        let mut result = 0;
        result |= self.input as u32;
        result |= (self.output as u32) << 8;
        result |= (self.initial as u32) << 16;
        result as usize
    }
}

/// Configures a GPIO.
pub fn configure(gpio: usize, config: &Config) -> Result<(), Error> {
    let params = api::configure::Params { gpio, mode: config.mode() };
    convert_unit(unsafe { api::configure(params) })
}

/// Reads from a GPIO (must be configured as input).
pub fn read(gpio: usize) -> Result<bool, Error> {
    let params = api::read::Params { gpio };
    convert_bool(unsafe { api::read(params) })
}

/// Writes to a GPIO (must be configured as output).
pub fn write(gpio: usize, value: bool) -> Result<(), Error> {
    let params = api::write::Params { gpio, val: value as usize };
    convert_unit(unsafe { api::write(params) })
}

/// Returns the last logical value written to a GPIO (must be configured as output).
pub fn last_write(gpio: usize) -> Result<bool, Error> {
    let params = api::last_write::Params { gpio };
    convert_bool(unsafe { api::last_write(params) })
}

/// GPIO that disconnects on drop.
pub struct Gpio(usize);

impl Drop for Gpio {
    fn drop(&mut self) {
        configure(self.0, &Config::disconnected()).unwrap();
    }
}

impl Gpio {
    /// Configures the GPIO and returns a handle.
    pub fn new(gpio: usize, config: &Config) -> Result<Self, Error> {
        configure(gpio, config)?;
        Ok(Gpio(gpio))
    }

    /// Reads from the GPIO (must be configured as input).
    pub fn read(&self) -> Result<bool, Error> {
        read(self.0)
    }

    /// Writes to the GPIO (must be configured as output).
    pub fn write(&self, value: bool) -> Result<(), Error> {
        write(self.0, value)
    }

    /// Returns the last logical value written to a GPIO (must be configured as output).
    pub fn last_write(&self) -> Result<bool, Error> {
        last_write(self.0)
    }

    /// Toggles the logical value of a GPIO (must be configured as output).
    pub fn toggle(&self) -> Result<(), Error> {
        self.write(!self.last_write()?)
    }
}

/// Provides callback support for GPIO events.
pub trait Handler: 'static {
    /// Called when a GPIO event triggered.
    fn event(&self);
}

impl<F: Fn() + 'static> Handler for F {
    fn event(&self) {
        self()
    }
}

/// Provides listening support for GPIO events.
#[must_use]
pub struct Listener<H: Handler> {
    gpio: usize,
    // Safety: This is a `Box<H>` we own and lend the platform as a shared borrow `&'a H` where
    // `'a` starts at `api::register()` and ends at `api::unregister()`.
    handler: *const H,
}

impl<H: Handler> Listener<H> {
    /// Starts listening for GPIO events.
    ///
    /// The `gpio` argument is the index of the button to listen events for. It must be less than
    /// [count()]. The `handler` argument is the callback to be called on events. Note that it may
    /// be an `Fn()` closure, see [Handler::event()] for callback documentation.
    ///
    /// The listener stops listening when dropped.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// Listener::new(index, Event::AnyChange, || debug!("An event triggered"))?
    /// ```
    pub fn new(gpio: usize, event: Event, handler: H) -> Result<Self, Error> {
        let handler_func = Self::call;
        let handler = Box::into_raw(Box::new(handler));
        let handler_data = handler as *const u8;
        let event = event as usize;
        let params = api::register::Params { gpio, event, handler_func, handler_data };
        convert_unit(unsafe { api::register(params) })?;
        Ok(Listener { gpio, handler })
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
        let params = api::unregister::Params { gpio: self.gpio };
        convert_unit(unsafe { api::unregister(params) }).unwrap();
        // SAFETY: `self.handler` is a `Box<H>` we own back, now that the lifetime of the platform
        // borrow is over (see `Listener::handler`).
        drop(unsafe { Box::from_raw(self.handler as *mut H) });
    }
}
