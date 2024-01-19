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

#![no_std]
wasefire::applet!();

use crate::serial::Serial;

mod serial;

fn main() {
    let uart = wasefire::uart::Uart::new(0).unwrap();
    let mut serial = Serial::new(&uart);
    loop {
        let request = serial.receive();
        debug!("Received {request:02x?}.");
        let response = Err("not implemented yet".into());
        debug!("Sending {response:02x?}.");
        serial.send(response);
    }
}
