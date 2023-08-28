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

use crate::*;

pub(crate) fn new() -> Item {
    let docs = docs! {
        /// Debugging operations.
    };
    let name = "debug".into();
    let items = vec![
        item! {
            /// Prints a message to the debug output.
            ///
            /// If debug output is disabled then this is a no-op.
            fn println "dp" {
                /// The message to print.
                ///
                /// Traps if the message is not valid UTF-8.
                ptr: *const u8,

                /// The length of the message in bytes.
                len: usize,
            } -> {}
        },
        item! {
            /// Returns the time spent since some initial event.
            ///
            /// The time is in micro-seconds and may wrap before using all 64 bits. In particular,
            /// this function may constantly return zero if time is not supported.
            fn time "dt" {
                /// Pointer to the upper 32-bits (may be null).
                ptr: *mut usize,
            } -> {
                /// Lower 32-bits of the time.
                res: usize,
            }
        },
        item! {
            /// Exits the platform with an error code.
            ///
            /// This is used by test applets to terminate the platform and propagate the test
            /// result.
            fn exit "de" {
                /// 0 for success, 1 for failure
                code: usize,
            } -> {}
        },
    ];
    Item::Mod(Mod { docs, name, items })
}
