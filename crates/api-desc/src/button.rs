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
        /// Button and touch operations.
    };
    let name = "button".into();
    let items = vec![
        item! {
            /// Describes the state of a button.
            enum State {
                /// The button is released.
                Released,
                /// The button is pressed.
                Pressed,
            }
        },
        item! {
            /// Returns how many buttons are on the device.
            fn count "bc" {} -> {
                /// How many buttons are on the device.
                cnt: usize,
            }
        },
        item! {
            /// Register a handler for button events.
            fn register "br" {
                /// Index of the button to listen to.
                button: usize,

                /// Function called on button events.
                ///
                /// The function takes its opaque `data` and the new button `state` as arguments.
                handler_func: fn { data: *mut u8, state: usize },

                /// The opaque data to use when calling the handler function.
                handler_data: *mut u8,
            } -> {}
        },
        item! {
            /// Unregister handlers for button events.
            fn unregister "bu" {
                /// Index of the button to stop listening to.
                button: usize,
            } -> {}
        },
    ];
    Item::Mod(Mod { docs, name, items })
}
