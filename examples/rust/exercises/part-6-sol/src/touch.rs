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

use wasefire::{button, clock, led, scheduling};

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
        });
        let timed_out = Rc::new(Cell::new(false));
        let timer = clock::Timer::new({
            let timed_out = timed_out.clone();
            move || timed_out.set(true)
        });
        let blink = clock::Timer::new(|| led::set(0, !led::get(0)));
        timer.start(clock::Oneshot, Duration::from_secs(5));
        blink.start(clock::Periodic, Duration::from_millis(200));
        scheduling::wait_until(|| pressed.get() || timed_out.get());
        led::set(0, led::Off);
        match pressed.get() {
            true => Ok(()),
            false => Err("touch timeout")?,
        }
    }
}
