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
        /// Clock and timer operations.
    };
    let name = "clock".into();
    let items = vec![
        item! {
            /// Whether a timer should periodically trigger.
            enum Mode {
                /// The timer fires only once.
                Oneshot = 0,

                /// The timer fires periodically.
                Periodic = 1,
            }
        },
        item! {
            /// Allocates a timer (initially stopped).
            fn allocate "ta" {
                /// Function called when the timer triggers.
                handler_func: fn { data: *const u8 },

                /// The opaque data to use when calling the handler function.
                handler_data: *const u8,
            } -> {
                /// Identifier for this timer.
                id: usize,
            }
        },
        item! {
            /// Starts a stopped timer given its id.
            fn start "tb" {
                /// The identifier of the timer to start.
                ///
                /// It must come from an allocated timer that wasn't stopped.
                id: usize,

                /// Whether the timer should periodically fire.
                ///
                /// Valid values are defined by [`Mode`](super::Mode).
                mode: usize,

                /// How long until the timer triggers in milli-seconds.
                duration_ms: usize,
            } -> {}
        },
        item! {
            /// Stops a running timer given its id.
            ///
            /// Note that if the timer triggers while being stopped, the handler may still be
            /// called.
            fn stop "tc" {
                /// The identifier of the timer to start.
                id: usize,
            } -> {}
        },
        item! {
            /// Deallocates a stopped timer given its id.
            fn free "td" {
                /// The identifier of the timer to start.
                id: usize,
            } -> {}
        },
    ];
    Item::Mod(Mod { docs, name, items })
}
