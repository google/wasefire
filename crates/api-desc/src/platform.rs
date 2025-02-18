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

use crate::*;

#[cfg(feature = "api-platform-protocol")]
mod protocol;
#[cfg(feature = "api-platform-update")]
mod update;

pub(crate) fn new() -> Item {
    let docs = docs! {
        /// Platform operations.
    };
    let name = "platform".into();
    let items = vec![
        #[cfg(feature = "api-platform-protocol")]
        protocol::new(),
        #[cfg(feature = "api-platform-update")]
        update::new(),
        #[cfg(feature = "api-platform")]
        item! {
            /// Reads the serial of the platform.
            ///
            /// This is an [allocating function](crate#allocating-memory) returning the length.
            fn serial "ps" {
                /// Where to write the serial.
                ptr: *mut *mut u8,
            } -> usize
        },
        #[cfg(feature = "api-platform")]
        item! {
            /// Returns whether the running side of the platform is B.
            fn running_side "pb" {
            } -> bool
        },
        #[cfg(feature = "api-platform")]
        item! {
            /// Reads the version of the platform.
            ///
            /// This is an [allocating function](crate#allocating-memory) returning the length.
            fn version "pv" {
                /// Whether to return the version of the running side (1) or opposite side (0).
                running: u32,

                /// Where to write the version.
                ptr: *mut *mut u8,
            } -> usize
        },
        #[cfg(feature = "api-platform")]
        item! {
            /// Reboots the device (thus platform and applets).
            ///
            /// Does not return on success.
            fn reboot "pr" {} -> !
        },
    ];
    Item::Mod(Mod { docs, name, items })
}
