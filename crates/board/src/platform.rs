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

#[cfg(feature = "api-platform")]
use crate::Error;

#[cfg(feature = "api-platform-protocol")]
pub mod protocol;
#[cfg(feature = "api-platform-update")]
pub mod update;

/// Platform event.
#[derive(Debug, PartialEq, Eq)]
pub enum Event {
    /// Protocol event.
    #[cfg(feature = "api-platform-protocol")]
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
    #[cfg(feature = "api-platform-protocol")]
    type Protocol: protocol::Api;

    /// Platform update interface.
    #[cfg(feature = "api-platform-update")]
    type Update: update::Api;

    /// Writes the platform version and returns its length in bytes.
    ///
    /// Versions are comparable with lexicographical order.
    ///
    /// The returned length may be larger than the buffer length, in which case the buffer only
    /// contains a prefix of the version. You can use the [`version_helper()`] to copy and return
    /// appropriately given a full version.
    #[cfg(feature = "api-platform")]
    fn version(output: &mut [u8]) -> usize;

    /// Reboots the device (thus platform and applets).
    #[cfg(feature = "api-platform")]
    fn reboot() -> Result<!, Error>;
}

/// Platform protocol interface.
#[cfg(feature = "api-platform-protocol")]
pub type Protocol<B> = <super::Platform<B> as Api>::Protocol;

/// Platform update interface.
#[cfg(feature = "api-platform-update")]
pub type Update<B> = <super::Platform<B> as Api>::Update;

/// Copies the longest prefix of a full version and returns its length.
///
/// You can use this function to implement [`Api::version()`] given a full version.
pub fn version_helper(version: &[u8], output: &mut [u8]) -> usize {
    let len = core::cmp::min(version.len(), output.len());
    output[.. len].copy_from_slice(&version[.. len]);
    version.len()
}
