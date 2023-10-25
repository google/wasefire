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

//! This exercise implements the core logic storing the private keys in RAM.

#![no_std]
wasefire::applet!();

use crate::logic::Logic;
use crate::serial::Serial;

mod logic; // TODO: Implement this module.
mod serial;

fn main() {
    let serial = Serial::new(uart::Uart(0));
    let mut logic = Logic::new(); // NEW
    loop {
        let request = serial.receive();
        debug!("Received {request:02x?}.");
        let response = logic.process(request); // NEW
        debug!("Sending {response:02x?}.");
        serial.send(response);
    }
}
