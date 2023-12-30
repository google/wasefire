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

use alloc::rc::Rc;
use core::cell::Cell;
use core::time::Duration;

use opensk_lib::api::user_presence::UserPresenceError::{Fail, Timeout};
use opensk_lib::api::user_presence::{UserPresence, UserPresenceResult};
use wasefire::button::{self, Listener};
use wasefire::clock::{Oneshot, Periodic};
use wasefire::{clock, debug, led, scheduling};

use crate::WasefireEnv;

impl UserPresence for WasefireEnv {
    fn check_init(&mut self) {}

    fn wait_with_timeout(&mut self, timeout_ms: usize) -> UserPresenceResult {
        // Make sure there is at least one button and one LED.
        let num_buttons = button::count();
        let num_leds = led::count();
        if num_buttons == 0 || num_leds == 0 {
            return Err(Fail);
        }

        // Start listening for button presses.
        // TODO: Should we be checking for any button pressed or only index 0?
        let button_pressed = Rc::new(Cell::new(false));
        for index in 0 .. num_buttons {
            let button_pressed = button_pressed.clone();
            let handler = move |state| match state {
                button::Pressed => button_pressed.set(true),
                button::Released => (),
            };
            let listener = Listener::new(index, handler);
            listener.leak();
        }

        // Start a periodic timer blinking the LED.
        // TODO: Similarly, should we be blinking all LEDs or only index 0?
        let blinking = Rc::new(Cell::new(false));
        let blink_timer = clock::Timer::new({
            let blinking = blinking.clone();
            move || {
                if blinking.get() {
                    led::set(0, !led::get(0));
                }
            }
        });
        blink_timer.start(Periodic, Duration::from_millis(200));

        // Start a timer for timeout.
        let timeout = Cell::new(false);
        let timeout_timer = clock::Timer::new({
            let timeout = timeout.clone();
            move || timeout.set(true)
        });
        timeout_timer.start(Oneshot, Duration::from_millis(timeout_ms as u64));

        // Wait until button press or timeout.
        scheduling::wait_until(|| button_pressed.get() || timeout.get());

        // Drop the periodic timer, then set the LED off.
        drop(blink_timer);
        blinking.set(false);

        if !button_pressed.get() {
            return Err(Timeout);
        }
        Ok(())
    }

    fn check_complete(&mut self) {}
}
