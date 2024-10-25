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

//! Platform protocol interface.

use alloc::boxed::Box;

use crate::Error;

/// Request received.
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, PartialEq, Eq)]
pub struct Event;

impl<B: crate::Api> From<Event> for crate::Event<B> {
    fn from(event: Event) -> Self {
        super::Event::Protocol(event).into()
    }
}

/// Platform protocol interface.
pub trait Api {
    /// Reads the last request, if any.
    fn read() -> Result<Option<Box<[u8]>>, Error>;

    /// Writes a response for the last request.
    fn write(response: &[u8]) -> Result<(), Error>;

    /// Starts accepting requests.
    ///
    /// Also triggers an event each time a request is received.
    fn enable() -> Result<(), Error>;

    /// Handles vendor-specific requests.
    fn vendor(request: &[u8]) -> Result<Box<[u8]>, Error>;
}
