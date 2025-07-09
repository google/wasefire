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

//! Low-level GPIO interface.

use derive_where::derive_where;

use crate::{Error, Id, Support};

/// GPIO event.
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive_where(Debug, PartialEq, Eq)]
pub struct Event<B: crate::Api + ?Sized> {
    /// The GPIO that triggered the event.
    pub gpio: Id<crate::Gpio<B>>,
}

impl<B: crate::Api> From<Event<B>> for crate::Event<B> {
    fn from(event: Event<B>) -> Self {
        crate::Event::Gpio(event)
    }
}

/// Input GPIO configuration.
#[derive(Debug, Copy, Clone, bytemuck::CheckedBitPattern)]
#[repr(u8)]
pub enum InputConfig {
    /// Input is disabled.
    Disabled = 0,

    /// Input is floating.
    Floating = 1,

    /// Input has a pull-down resistor.
    PullDown = 2,

    /// Input has a pull-up resistor.
    PullUp = 3,
}

/// Output GPIO configuration.
#[derive(Debug, Copy, Clone, bytemuck::CheckedBitPattern)]
#[repr(u8)]
pub enum OutputConfig {
    /// Output is disabled.
    Disabled = 0,

    /// Output can only drive low.
    OpenDrain = 1,

    /// Output can only drive high.
    OpenSource = 2,

    /// Output can drive both low and high.
    PushPull = 3,
}

/// GPIO configuration.
#[derive(Debug, Copy, Clone, bytemuck::CheckedBitPattern)]
#[repr(C)]
pub struct Config {
    /// Input configuration.
    pub input: InputConfig,

    /// Output configuration.
    pub output: OutputConfig,

    /// Initial output value.
    pub initial: bool,

    reserved: u8,
}

/// Low-level GPIO interface.
pub trait Api: Support<usize> + Send {
    /// Configures a GPIO.
    fn configure(gpio: Id<Self>, config: Config) -> Result<(), Error>;

    /// Reads from a GPIO (must be configured as input).
    fn read(gpio: Id<Self>) -> Result<bool, Error>;

    /// Writes to a GPIO (must be configured as output).
    fn write(gpio: Id<Self>, value: bool) -> Result<(), Error>;

    /// Returns the last logical value written to a GPIO (must be configured as output).
    fn last_write(gpio: Id<Self>) -> Result<bool, Error>;

    /// Enables an event to be triggered on the given conditions (at least one must be true).
    fn enable(gpio: Id<Self>, falling: bool, rising: bool) -> Result<(), Error>;

    /// Disables an event from being triggered on all conditions.
    fn disable(gpio: Id<Self>) -> Result<(), Error>;
}
