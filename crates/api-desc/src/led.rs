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
        /// LED operations.
    };
    let name = "led".into();
    let items = vec![
        item! {
            /// Returns how many LEDs are on the device.
            fn count "lc" {} -> {
                /// How many LEDs are on the device.
                cnt: usize,
            }
        },
        item! {
            /// Describes the state of a LED.
            enum Status {
                /// The LED is off.
                Off,

                /// The LED is on.
                On,
            }
        },
        item! {
            /// Returns a LED status.
            fn get "lg" {
                /// Index of the LED to set.
                led: usize,
            } -> {
                /// 0 for off and 1 for on.
                status: usize,
            }
        },
        item! {
            /// Sets a LED status.
            fn set "ls" {
                /// Index of the LED to set.
                led: usize,

                /// 0 for off and 1 for on.
                status: usize,
            } -> {}
        },
    ];
    Item::Mod(Mod { docs, name, items })
}
