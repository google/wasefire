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
        /// Bluetooth Low Energy (BLE) operations.
    };
    let name = "ble".into();
    let items = vec![
        item! {
            /// BLE events.
            enum Event {
                /// Advertisement packets.
                Advertisement = 0,
            }
        },
        item! {
            /// Advertisement packet.
            struct Advertisement {
                ticks: u32,
                freq: u16,
                rssi: i8,
                pdu_type: u8,
                addr: [6; u8],
                data_len: u8,
                data: [31; u8],
                _padding: [2; u8],
            }
        },
        item! {
            /// Reads the next advertisement packet into a buffer, if any.
            fn read_advertisement "rlra" {
                /// Pointer to the [`super::Advertisement`] packet.
                ptr: *mut u8,
            } -> {
                /// One if a packet was read. Zero if there was no packet to read. Otherwise
                /// complement of error number.
                res: isize,
            }
        },
        item! {
            /// Register a handler for radio events.
            fn register "rle" {
                /// Radio [`super::Event`] to listen to.
                event: u32,

                /// Function called on radio events.
                ///
                /// The function takes its opaque `data` as argument.
                handler_func: fn { data: *const void },

                /// The opaque data to use when calling the handler function.
                handler_data: *const void,
            } -> {}
        },
        item! {
            /// Unregister handlers for radio events.
            fn unregister "rld" {
                /// Radio [`super::Event`] to stop listening to.
                event: u32,
            } -> {}
        },
    ];
    Item::Mod(Mod { docs, name, items })
}
