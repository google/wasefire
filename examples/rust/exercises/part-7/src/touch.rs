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
use core::cell::{Cell, RefCell};
use core::time::Duration;

use wasefire::{button, debug, led, scheduling, timer};

#[derive(Default, Clone)]
pub struct Touch(Rc<RefCell<Option<TouchImpl>>>);

struct TouchImpl {
    // TODO: Add a boolean to know whether there is a touch ready to be consumed.
    // TODO: Store the always listening button::Listener. You need to implement button::Handler for
    // Touch.
    // TODO: Store the timer for blinking the LED. You may create a unit struct Blink and implement
    // timer::Handler for it (toggling the LED).
    // TODO: Store the timer for timeout (either the timeout of a button touch until consumed, or
    // the timeout waiting for a touch). You need to implement timer::Handler for Touch.
}

impl Touch {
    pub fn new() -> Self {
        // TODO: Allocate a default Touch (a reference pointing to None).
        // TODO: Create a TouchImpl.
        // TODO: Update the Touch with Some of the created TouchImpl.
        todo!()
    }

    pub fn consume(&self) -> Result<(), String> {
        // TODO: Check whether the button was touched. If yes, consume it and return.
        // TODO: Otherwise, wait for a touch.
        // TODO: Start blinking.
        // TODO: Start timeout.
        // TODO: Wait until touched or timed out.
        todo!()
    }
}
