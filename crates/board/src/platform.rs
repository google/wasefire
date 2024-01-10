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

#[cfg(feature = "api-platform-update")]
pub mod update;

/// Platform interface.
pub trait Api: Send {
    #[cfg(feature = "api-platform-update")]
    type Update: update::Api;

    /// Writes the platform version and returns its length in bytes.
    ///
    /// Versions are comparable with lexicographical order.
    ///
    /// The returned length may be larger than the buffer length, in which case the buffer only
    /// contains a prefix of the version.
    #[cfg(feature = "api-platform")]
    fn version(output: &mut [u8]) -> usize;

    /// Reboots the device (thus platform and applets).
    #[cfg(feature = "api-platform")]
    fn reboot() -> Result<!, Error>;
}

#[cfg(feature = "api-platform-update")]
pub type Update<B> = <super::Platform<B> as Api>::Update;
