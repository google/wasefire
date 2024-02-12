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

use wasefire_applet_api::gpio as api;
pub use wasefire_applet_api::gpio::{InputConfig, OutputConfig};

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
pub fn configure(gpio: usize, config: &Config) {
    let params = api::configure::Params { gpio, mode: config.mode() };
    convert_unit(unsafe { api::configure(params) }).unwrap();
}

/// Reads from a GPIO (must be configured as input).
pub fn read(gpio: usize) -> bool {
    let params = api::read::Params { gpio };
    convert_bool(unsafe { api::read(params) }).unwrap()
}

/// Writes to a GPIO (must be configured as output).
pub fn write(gpio: usize, value: bool) {
    let params = api::write::Params { gpio, val: value as usize };
    convert_unit(unsafe { api::write(params) }).unwrap();
}

/// Returns the last logical value written to a GPIO (must be configured as output).
pub fn last_write(gpio: usize) -> bool {
    let params = api::last_write::Params { gpio };
    convert_bool(unsafe { api::last_write(params) }).unwrap()
}

/// GPIO that disconnects on drop.
pub struct Gpio(usize);

impl Drop for Gpio {
    fn drop(&mut self) {
        configure(self.0, &Config::disconnected());
    }
}

impl Gpio {
    /// Configures the GPIO and returns a handle.
    pub fn new(gpio: usize, config: &Config) -> Self {
        configure(gpio, config);
        Gpio(gpio)
    }

    /// Reads from the GPIO (must be configured as input).
    pub fn read(&self) -> bool {
        read(self.0)
    }

    /// Writes to the GPIO (must be configured as output).
    pub fn write(&self, value: bool) {
        write(self.0, value)
    }

    /// Returns the last logical value written to a GPIO (must be configured as output).
    pub fn last_write(&self) -> bool {
        last_write(self.0)
    }

    /// Toggles the logical value of a GPIO (must be configured as output).
    pub fn toggle(&self) {
        self.write(!self.last_write());
    }
}
