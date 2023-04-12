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

//! Button interface.
//!
//! A button is an input interface with 2 states: pressed and released. Buttons must support
//! triggering events when changing state. Events may be enabled or disabled per button.

use crate::Error;

/// Button event.
#[derive(Debug, PartialEq, Eq)]
pub struct Event {
    /// The button that triggered the event.
    pub button: usize,

    /// Whether the event was a button press or release.
    pub pressed: bool,
}

impl From<Event> for crate::Event {
    fn from(event: Event) -> Self {
        crate::Event::Button(event)
    }
}

/// Button interface.
pub trait Api {
    /// Returns how many buttons are available.
    ///
    /// Buttons are identified by an integer smaller than this value.
    fn count(&mut self) -> usize;

    /// Enables events for a given button.
    fn enable(&mut self, button: usize) -> Result<(), Error>;

    /// Disables events for a given button.
    fn disable(&mut self, button: usize) -> Result<(), Error>;
}

impl Api for ! {
    fn count(&mut self) -> usize {
        unreachable!()
    }

    fn enable(&mut self, _: usize) -> Result<(), Error> {
        unreachable!()
    }

    fn disable(&mut self, _: usize) -> Result<(), Error> {
        unreachable!()
    }
}

impl Api for () {
    fn count(&mut self) -> usize {
        0
    }

    fn enable(&mut self, _: usize) -> Result<(), Error> {
        Err(Error::User)
    }

    fn disable(&mut self, _: usize) -> Result<(), Error> {
        Err(Error::User)
    }
}
