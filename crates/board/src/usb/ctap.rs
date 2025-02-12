// Copyright 2024 Google LLC
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

//! CTAP HID interface.

use crate::Error;

/// CTAP HID event.
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, PartialEq, Eq)]
pub enum Event {
    /// There might be data to read.
    ///
    /// This is only guaranteed to be triggered after a failed read.
    Read,

    /// It might be possible to write data.
    ///
    /// This is only guaranteed to be triggered after a failed write.
    Write,
}

impl<B: crate::Api> From<Event> for crate::Event<B> {
    fn from(event: Event) -> Self {
        super::Event::Ctap(event).into()
    }
}

/// CTAP HID interface.
pub trait Api: Send {
    /// Reads a CTAP HID packet into a buffer.
    ///
    /// Returns whether a packet was read read.
    fn read(output: &mut [u8; 64]) -> Result<bool, Error>;

    /// Writes a CTAP HID packet from a buffer.
    ///
    /// Returns whether the packet was written.
    fn write(input: &[u8; 64]) -> Result<bool, Error>;

    /// Enables a given event to be triggered.
    fn enable(event: &Event) -> Result<(), Error>;

    /// Disables a given event from being triggered.
    fn disable(event: &Event) -> Result<(), Error>;
}
