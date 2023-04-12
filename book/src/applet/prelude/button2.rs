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

//! Demonstrates simple LED and button interaction.
//!
//! The board must possess at least one LED and one button. The applet maintains a dynamic mapping
//! from buttons to LEDs so that every button may control any LED over time. Initially, all buttons
//! are associated to the first LED. Each time a button is pressed or released, its associated LED
//! is toggled. After a button is released, it becomes associated with the next LED (wrapping back
//! to the first LED after the last LED).

//{ ANCHOR: all
#![no_std]
wasefire::applet!();

use alloc::boxed::Box;
use core::cell::Cell;

fn main() {
    //{ ANCHOR: setup
    // Make sure there is at least one button.
    let num_buttons = button::count();
    assert!(num_buttons > 0, "Board has no buttons.");

    // Make sure there is at least one LED.
    let num_leds = led::count();
    assert!(num_leds > 0, "Board has no LEDs.");

    // For each button on the board.
    for button_index in 0 .. num_buttons {
    //} ANCHOR_END: setup
        //{ ANCHOR: state
        // We create the state containing the LED to which this button maps to.
        let led_pointer = Box::new(Cell::new(0));
        //} ANCHOR_END: state

        //{ ANCHOR: handler
        // We define the button handler and move the state to there.
        let handler = move |button_state| {
        //} ANCHOR_END: handler
            //{ ANCHOR: toggle
            // We toggle the LED.
            let led_index = led_pointer.get();
            led::set(led_index, !led::get(led_index));
            //} ANCHOR_END: toggle

            //{ ANCHOR: update
            // If the button is released, we point it to the next LED.
            if matches!(button_state, button::Released) {
                led_pointer.set((led_index + 1) % num_leds);
            }
            //} ANCHOR_END: update
        };

        //{ ANCHOR: listener
        // We indefinitely listen by creating and leaking a listener.
        button::Listener::new(button_index, handler).leak();
        //} ANCHOR_END: listener
    }

    // We indefinitely wait for callbacks.
    scheduling::wait_indefinitely();
}
//} ANCHOR_END: all
