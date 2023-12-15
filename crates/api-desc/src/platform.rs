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

mod update;

pub(crate) fn new() -> Item {
    let docs = docs! {
        /// Platform operations.
    };
    let name = "platform".into();
    let items = vec![
        update::new(),
        item! {
            /// Returns the version of the platform.
            fn version "pv" {
                /// Where to write the version.
                ptr: *mut u8,

                /// Capacity of the buffer.
                len: usize,
            } -> {
                /// Length of the version in bytes.
                ///
                /// This may be larger than the capacity, in which case only a prefix was written.
                len: usize,
            }
        },
        item! {
            /// Reboots the device (thus platform and applets).
            fn reboot "pr" {} -> {
                /// Complement of error number.
                res: isize,
            }
        },
    ];
    Item::Mod(Mod { docs, name, items })
}
