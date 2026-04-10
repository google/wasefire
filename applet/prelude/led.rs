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

//! Demonstrates simple LED usage.
//!
//! The board must possess at least one LED and one timer. The applet indefinitely cycles through
//! all LEDs in order with a period of 1 second and blinks them for 500 milli-seconds.

//{ ANCHOR: all
#![no_std]
wasefire::applet!();

use core::time::Duration;

fn main() {
    //{ ANCHOR: count
    // Make sure there is at least one LED.
    let num_leds = led::count();
    assert!(num_leds > 0, "Board has no LEDs.");
    //} ANCHOR_END: count

    //{ ANCHOR: loop
    // Cycle indefinitely through all LEDs in order.
    for led_index in (0 .. num_leds).cycle() {
    //} ANCHOR_END: loop
        //{ ANCHOR: set
        // Turn on the current LED.
        led::set(led_index, led::On);
        //} ANCHOR_END: set
        //{ ANCHOR: sleep
        // Wait before turning it off.
        timer::sleep(Duration::from_millis(500));
        //} ANCHOR_END: sleep

        //{ ANCHOR: repeat
        // Turn it off and wait before turning on the next LED.
        led::set(led_index, led::Off);
        timer::sleep(Duration::from_millis(500));
        //} ANCHOR_END: repeat
    }
}
//} ANCHOR_END: all
