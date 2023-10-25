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

use wasefire::{button, led, scheduling};

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
        scheduling::wait_until(|| pressed.get());
        led::set(0, led::Off);
        Ok(())
    }
}
