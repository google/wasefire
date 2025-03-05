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

//! Platform interface.

use crate::Error;

pub mod protocol;
pub mod update;

/// Platform event.
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, PartialEq, Eq)]
pub enum Event {
    /// Protocol event.
    Protocol(protocol::Event),
}

impl<B: crate::Api> From<Event> for crate::Event<B> {
    fn from(event: Event) -> Self {
        crate::Event::Platform(event)
    }
}

/// Platform interface.
pub trait Api: Send {
    /// Platform protocol interface.
    type Protocol: protocol::Api;

    /// Platform update interface.
    type Update: update::Api;

    /// Returns the platform serial.
    fn serial() -> alloc::borrow::Cow<'static, [u8]>;

    /// Returns the platform running side.
    fn running_side() -> wasefire_common::platform::Side;

    /// Returns the platform running version.
    ///
    /// Versions are comparable with lexicographical order.
    fn running_version() -> alloc::borrow::Cow<'static, [u8]>;

    /// Returns the version of the opposite side, if any.
    ///
    /// # Errors
    ///
    /// There is a precise meaning to the following errors:
    /// - `World:NotEnough`: This platform has only one side.
    /// - `World:NotFound`: The other side is empty.
    fn opposite_version() -> Result<alloc::borrow::Cow<'static, [u8]>, Error>;

    /// Reboots the device (thus platform and applets).
    fn reboot() -> Result<!, Error>;
}

/// Platform protocol interface.
pub type Protocol<B> = <super::Platform<B> as Api>::Protocol;

/// Platform update interface.
pub type Update<B> = <super::Platform<B> as Api>::Update;
