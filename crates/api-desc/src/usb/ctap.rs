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
    let docs = docs!();
    let name = "ctap".into();
    let items = vec![
        item! {
            /// Reads from CTAP HID into a buffer.
            ///
            /// This function does not block and returns whether a packet was read.
            fn read "ucr" {
                /// Address of the 64-bytes buffer.
                ptr: *mut u8,
            } -> bool
        },
        item! {
            /// Writes to CTAP HID from a buffer.
            ///
            /// This function does not block and returns if the packet was written.
            fn write "ucw" {
                /// Address of the 64-bytes buffer.
                ptr: *const u8,
            } -> bool
        },
        item! {
            /// CTAP HID events.
            enum Event {
                /// Ready for read.
                Read = 0,
                /// Ready for write.
                Write = 1,
            }
        },
        item! {
            /// Registers a callback when CTAP HID is ready.
            ///
            /// It is possible that the callback is spuriously called. The callback is only
            /// guaranteed to be called after the associated operation returned false.
            fn register "uce" {
                event: usize,
                handler_func: fn { data: *const void },
                handler_data: *const void,
            } -> ()
        },
        item! {
            /// Unregisters a callback.
            fn unregister "ucd" {
                event: usize,
            } -> ()
        },
    ];
    Item::Mod(Mod { docs, name, items })
}
