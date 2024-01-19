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
use wasefire::timer::{Oneshot, Periodic};
use wasefire::{led, scheduling, timer};

use crate::WasefireEnv;

impl UserPresence for WasefireEnv {
    fn check_init(&mut self) {}

    fn wait_with_timeout(&mut self, timeout_ms: usize) -> UserPresenceResult {
        // Make sure there is at least one button and one LED.
        if button::count() == 0 || led::count() == 0 {
            return Err(Fail);
        }

        // Start listening for button presses.
        let button_pressed = Rc::new(Cell::new(false));
        let listener = Listener::new(0, {
            let button_pressed = button_pressed.clone();
            move |state| match state {
                button::Pressed => button_pressed.set(true),
                button::Released => (),
            }
        });

        // Start a periodic timer blinking the LED.
        let blink_timer = timer::Timer::new(|| {
            led::set(0, !led::get(0));
        });
        led::set(0, led::On);
        blink_timer.start(Periodic, Duration::from_millis(200));

        // Start a timer for timeout.
        let timeout = Cell::new(false);
        let timeout_timer = timer::Timer::new({
            let timeout = timeout.clone();
            move || timeout.set(true)
        });
        timeout_timer.start(Oneshot, Duration::from_millis(timeout_ms as u64));

        // Wait until button press or timeout.
        scheduling::wait_until(|| button_pressed.get() || timeout.get());

        // Drop the periodic timer, then set the LED off.
        drop(blink_timer);
        led::set(0, led::Off);

        if !button_pressed.get() {
            return Err(Timeout);
        }
        Ok(())
    }

    fn check_complete(&mut self) {}
}
