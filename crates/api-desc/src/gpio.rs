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
            fn count "gc" {} -> usize
        },
        item! {
            /// Input configuration.
            enum InputConfig {
                /// Input is disabled.
                Disabled = 0,

                /// Floating input (most common configuration).
                ///
                /// Reading while the voltage is not driven may return 0 or 1.
                Floating = 1,

                /// Pull-down input.
                ///
                /// Reading while the voltage is not driven returns 0.
                PullDown = 2,

                /// Pull-up input.
                ///
                /// Reading while the voltage is not driven returns 1.
                PullUp = 3,
            }
        },
        item! {
            /// Output configuration.
            enum OutputConfig {
                /// Output is disabled.
                Disabled = 0,

                /// Push-pull output (most common configuration).
                ///
                /// Writing 0 (resp. 1) drives the voltage to 0 (resp. 1).
                PushPull = 3,

                /// Open-drain output.
                ///
                /// Writing 0 drives the voltage to 0. Writing 1 doesn't drive the voltage.
                OpenDrain = 1,

                /// Open-source output.
                ///
                /// Writing 0 doesn't drive the voltage. Writing 1 drives the voltage to 1.
                OpenSource = 2,
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
            } -> ()
        },
        item! {
            /// Reads from a GPIO.
            fn read "gr" {
                /// Index of the GPIO to read from (must be configured as input).
                gpio: usize,
            } -> bool
        },
        item! {
            /// Writes to a GPIO.
            fn write "gw" {
                /// Index of the GPIO to write to (must be configured as output).
                gpio: usize,

                /// Logical value (0 or 1).
                val: usize,
            } -> ()
        },
        item! {
            /// Returns the last logical value written to a GPIO.
            ///
            /// The initial output value counts as a write and would be returned if `write()` was
            /// not called since last `configure()`.
            fn last_write "gl" {
                /// Index of the GPIO to query (must be configured as output).
                gpio: usize,
            } -> bool
        },
        item! {
            /// GPIO events.
            enum Event {
                FallingEdge = 0,
                RisingEdge = 1,
                AnyChange = 2,
            }
        },
        item! {
            /// Register a handler for gpio events.
            fn register "gg" {
                /// Index of the gpio to listen to.
                gpio: usize,

                /// Event to listen to.
                event: usize,

                /// Function called on gpio events.
                handler_func: fn { data: *const void },

                /// The opaque data to use when calling the handler function.
                handler_data: *const void,
            } -> ()
        },
        item! {
            /// Unregister handlers for gpio events.
            fn unregister "gu" {
                /// Index of the gpio to stop listening to.
                gpio: usize,
            } -> ()
        },
    ];
    Item::Mod(Mod { docs, name, items })
}
