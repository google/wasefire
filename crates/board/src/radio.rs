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

//! Radio interface.

use crate::{Error, Support, Unsupported};

/// Radio event.
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Event {
    /// A radio packet has been received
    Received,
}

impl<B: crate::Api> From<Event> for crate::Event<B> {
    fn from(event: Event) -> Self {
        crate::Event::Radio(event)
    }
}

/// Radio interface.
pub trait Api: Support<bool> {
    /// Enables radio events
    fn enable() -> Result<(), Error>;

    /// Disables radio events
    fn disable() -> Result<(), Error>;

    /// Reads from the radio receive queue into a buffer.
    ///
    /// Returns the number of bytes read. It could be zero if there's nothing to read.
    fn read(output: &mut [u8]) -> Result<usize, Error>;
}

impl Api for Unsupported {
    fn enable() -> Result<(), Error> {
        unreachable!()
    }

    fn disable() -> Result<(), Error> {
        unreachable!()
    }

    fn read(_: &mut [u8]) -> Result<usize, Error> {
        unreachable!()
    }
}
