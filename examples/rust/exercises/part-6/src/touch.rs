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
use alloc::string::String;
use core::cell::Cell;
use core::time::Duration;

use wasefire::{button, led, scheduling, timer};

#[derive(Default, Clone)]
pub struct Touch;

impl Touch {
    pub fn new() -> Self {
        Touch
    }

    pub fn consume(&self) -> Result<(), String> {
        led::set(0, led::On);
        let pressed = Rc::new(Cell::new(false));
        let _button = button::Listener::new(0, {
            let pressed = pressed.clone();
            move |state| {
                if matches!(state, button::Pressed) {
                    pressed.set(true);
                }
            }
        })
        .unwrap();
        // TODO: Create a timed_out variable similar to pressed.
        // TODO: Use timer::Timer to allocate a new timer.
        // TODO: Unconditionally set the timed_out variable in its callback (similar to pressed).
        // TODO(optional): Allocate another timer to blink the LED. The callback should toggle the
        // LED using led::get() and negating its value.
        // TODO: Start the timer for 5 seconds using the start() method and timer::Oneshot.
        // TODO(optional): Start the blinking timer to trigger every 200 milliseconds using
        // timer::Periodic.
        // TODO: Update the waiting condition to also return when timed_out is set.
        scheduling::wait_until(|| pressed.get());
        led::set(0, led::Off);
        // TODO: Only return Ok(()) when pressed is set. Otherwise return a touch timeout error.
        Ok(())
    }
}
