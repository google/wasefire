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

//! LED interface.
//!
//! A LED is an output interface with 2 states: on and off.

use crate::Error;

/// LED interface.
pub trait Api {
    /// Returns how many LEDs are available.
    ///
    /// LEDs are identified by an integer smaller than this value.
    fn count(&mut self) -> usize;

    /// Returns whether a given LED is on.
    fn get(&mut self, led: usize) -> Result<bool, Error>;

    /// Sets the state of a given LED.
    fn set(&mut self, led: usize, on: bool) -> Result<(), Error>;
}

impl Api for ! {
    fn count(&mut self) -> usize {
        unreachable!()
    }

    fn get(&mut self, _: usize) -> Result<bool, Error> {
        unreachable!()
    }

    fn set(&mut self, _: usize, _: bool) -> Result<(), Error> {
        unreachable!()
    }
}

impl Api for () {
    fn count(&mut self) -> usize {
        0
    }

    fn get(&mut self, _: usize) -> Result<bool, Error> {
        Err(Error::User)
    }

    fn set(&mut self, _: usize, _: bool) -> Result<(), Error> {
        Err(Error::User)
    }
}
