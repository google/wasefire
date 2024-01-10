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

use crate::{Error, Id, Support};

#[derive(Debug, Copy, Clone, bytemuck::CheckedBitPattern)]
#[repr(u8)]
pub enum InputConfig {
    Disabled = 0,
    Floating = 1,
    PullDown = 2,
    PullUp = 3,
}

#[derive(Debug, Copy, Clone, bytemuck::CheckedBitPattern)]
#[repr(u8)]
pub enum OutputConfig {
    Disabled = 0,
    OpenDrain = 1,
    OpenSource = 2,
    PushPull = 3,
}

#[derive(Debug, Copy, Clone, bytemuck::CheckedBitPattern)]
#[repr(C)]
pub struct Config {
    pub input: InputConfig,
    pub output: OutputConfig,
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
}
