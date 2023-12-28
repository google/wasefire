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
        /// Low-level GPIO operations.
        ///
        /// See [`crate::button`] and [`crate::led`] for higher-level GPIO operations.
    };
    let name = "gpio".into();
    let items = vec![
        item! {
            /// Returns how many GPIOs are on the device.
            fn count "gc" {} -> {
                /// How many GPIOs are on the device.
                cnt: usize,
            }
        },
        item! {
            /// Input configuration.
            enum InputConfig {
                Disabled = 0,
                Floating = 1,
                PullDown = 2,
                PullUp = 3,
            }
        },
        item! {
            /// Output configuration.
            enum OutputConfig {
                Disabled = 0,
                OpenDrain = 1,
                OpenSource = 2,
                PushPull = 3,
            }
        },
        item! {
            /// Configures a GPIO.
            fn configure "gf" {
                /// Index of the GPIO to configure.
                gpio: usize,

                /// Bit-field describing the configuration.
                ///
                /// | Bits  | Description          |
                /// | ---   | ---                  |
                /// | 01:00 | Input configuration  |
                /// | 09:08 | Output configuration |
                /// | 16:16 | Output initial value |
                mode: usize,
            } -> {
                /// Zero. Negative on error.
                res: isize,
            }
        },
        item! {
            /// Reads from a GPIO.
            fn read "gr" {
                /// Index of the GPIO to read from (must be configured as input).
                gpio: usize,
            } -> {
                /// Zero or One. Negative on error.
                val: isize,
            }
        },
        item! {
            /// Writes to a GPIO.
            fn write "gw" {
                /// Index of the GPIO to write to (must be configured as output).
                gpio: usize,

                /// Zero or one.
                val: usize,
            } -> {
                /// Zero. Negative on error.
                res: isize,
            }
        },
    ];
    Item::Mod(Mod { docs, name, items })
}
