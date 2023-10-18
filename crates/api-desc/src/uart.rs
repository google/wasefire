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
    let docs = docs!();
    let name = "uart".into();
    let items = vec![
        item! {
            /// Returns how many UARTs are on the device.
            fn count "uac" {} -> { cnt: usize }
        },
        item! {
            /// Reads from a UART into a buffer.
            fn read "uar" {
                /// Index of the UART to read from.
                uart: usize,

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
            /// Writes to a UART from a buffer.
            fn write "uaw" {
                /// Index of the UART to write to.
                uart: usize,

                /// Address of the buffer.
                ptr: *const u8,

                /// Length of the buffer in bytes.
                len: usize,
            } -> {
                /// Number of bytes written (or negative value for errors).
                ///
                /// This function does not block and may return zero.
                len: isize,
            }
        },
        item! {
            /// UART events.
            enum Event {
                /// Ready for read.
                Read = 0,

                /// Ready for write.
                Write = 1,
            }
        },
        item! {
            /// Registers a callback when a UART is ready.
            ///
            /// It is possible that the callback is spuriously called.
            fn register "uae" {
                /// Index of the UART to listen to.
                uart: usize,

                /// Event to listen to.
                event: usize,

                /// Function pointer of the closure to call on events.
                handler_func: fn { data: *const u8 },

                /// Opaque data of the closure to call on events.
                handler_data: *const u8,
            } -> {}
        },
        item! {
            /// Unregisters a callback.
            fn unregister "uad" {
                /// Index of the UART to stop listening to.
                uart: usize,

                /// Event to stop listening to.
                event: usize,
            } -> {}
        },
    ];
    Item::Mod(Mod { docs, name, items })
}
