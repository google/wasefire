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
            /// Returns the serial of the platform.
            ///
            /// Returns the length of the serial in bytes. The serial is not allocated if the
            /// length is zero (and the pointer is not written).
            fn serial "ps" {
                /// Where to write the serial.
                ///
                /// If the returned length is positive, the (inner) pointer will be allocated by the
                /// callee and must be freed by the caller. It is thus owned by the caller when the
                /// function returns.
                ptr: *mut *mut u8,
            } -> usize
        },
        #[cfg(feature = "api-platform")]
        item! {
            /// Returns the version of the platform.
            ///
            /// Returns the length of the version in bytes. The version is not allocated if the
            /// length is zero (and the pointer is not written).
            fn version "pv" {
                /// Where to write the version.
                ///
                /// If the returned length is positive, the (inner) pointer will be allocated by the
                /// callee and must be freed by the caller. It is thus owned by the caller when the
                /// function returns.
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
