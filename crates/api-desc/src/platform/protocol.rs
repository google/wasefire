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

use crate::*;

pub(crate) fn new() -> Item {
    let docs = docs! {
        /// Platform protocol.
    };
    let name = "protocol".into();
    let items = vec![
        item! {
            /// Reads the last request, if any.
            ///
            /// Returns whether a request was allocated.
            fn read "ppr" {
                /// Where to write the request, if any.
                ///
                /// The (inner) pointer will be allocated by the callee and must be freed by the
                /// caller. It is thus owned by the caller when the function returns.
                ptr: *mut *mut u8,

                /// Where to write the length of the request, if any.
                len: *mut usize,
            } -> bool
        },
        item! {
            /// Writes a response to the last request.
            fn write "ppw" {
                /// Address of the response.
                ptr: *const u8,

                /// Length of the response in bytes.
                len: usize,
            } -> ()
        },
        item! {
            /// Registers a callback when a request is received.
            fn register "ppe" {
                handler_func: fn { data: *const u8 },
                handler_data: *const u8,
            } -> ()
        },
        item! {
            /// Unregisters the callback.
            fn unregister "ppd" {} -> ()
        },
    ];
    Item::Mod(Mod { docs, name, items })
}
