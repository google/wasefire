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

//! Demonstrates simple timer usage to cancel button press.
//!
//! The applet relies on LEDs, buttons, and timers. The applet uses the first button and first LED.
//! When the button is pressed, the LED turns on and a 1 second timer starts. If the timer runs out
//! before the button is released, then the LED turns off and releasing the button will do nothing.
//! If the button is released before the timer runs out, then the LED will blink until next button
//! press.

#![no_std]
wasefire::applet!();

use alloc::rc::Rc;
use core::cell::Cell;
use core::time::Duration;

fn main() {
    assert!(button::count() > 0, "Board has no buttons.");
    assert!(led::count() > 0, "Board has no LEDs.");

    // We define a shared state to decide whether we must blink.
    let blinking = Rc::new(Cell::new(false));

    // Allocate a timer for blinking the LED.
    let blink = timer::Timer::new({
        // Move a clone of the state to the callback.
        let blinking = blinking.clone();
        move || {
            // Toggle the LED if blinking.
            if blinking.get() {
                led::set(0, !led::get(0));
            }
        }
    });

    loop {
        // Setup button listeners.
        let pressed = Rc::new(Cell::new(false));
        let released = Rc::new(Cell::new(false));
        let _button = button::Listener::new(0, {
            let pressed = pressed.clone();
            let released = released.clone();
            move |state| match state {
                button::Pressed => pressed.set(true),
                button::Released if pressed.get() => released.set(true),
                button::Released => (),
            }
        });

        // Wait for the button to be pressed.
        scheduling::wait_until(|| pressed.get());

        // Turn the LED on.
        blink.stop();
        blinking.set(false);
        led::set(0, led::On);

        // Start the timeout to decide between short and long press.
        let timed_out = Rc::new(Cell::new(false));
        let timer = timer::Timer::new({
            let timed_out = timed_out.clone();
            move || timed_out.set(true)
        });
        timer.start(timer::Oneshot, Duration::from_secs(1));

        // Wait for the button to be released or the timeout to fire.
        scheduling::wait_until(|| released.get() || timed_out.get());

        // Turn the LED off.
        led::set(0, led::Off);

        // Start blinking if short press.
        if !timed_out.get() {
            blinking.set(true);
            blink.start(timer::Periodic, Duration::from_millis(200));
        }
    }
}
