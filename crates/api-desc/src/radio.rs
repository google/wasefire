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

pub(crate) fn new() -> Item {
    let docs = docs! {
        /// Radio operations.
    };
    let name = "radio".into();
    let items = vec![
        item! {
            /// Reads radio packet into a buffer.
            fn read "rr" {
                /// Address of the buffer.
                ptr: *mut u8,

                /// Length of the buffer in bytes.
                len: usize,
            } -> {
                /// Number of bytes read (or negative value for errors).
                ///
                /// This function does not block and may return zero.
                len: isize,
            }
        },
        item! {
            /// Register a handler for radio events.
            fn register "re" {
                /// Function called on radio events.
                ///
                /// The function takes its opaque `data` as argument.
                handler_func: fn { data: *mut u8 },

                /// The opaque data to use when calling the handler function.
                handler_data: *mut u8,
            } -> {}
        },
        item! {
            /// Unregister handlers for radio events.
            fn unregister "rd" {} -> {}
        },
        item! {
            /// Describes errors on radio operations.
            enum Error {
                Unknown = 0,
            }
        },
    ];
    Item::Mod(Mod { docs, name, items })
}
