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

fn main() {
    let serial = uart::Uart(0);
    loop {
        let mut buffer = [0; 32];
        let len = serial::read_any(&serial, &mut buffer).unwrap();
        let data = &buffer[.. len];
        debug!("Received {data:02x?}.");
        serial::write_all(&serial, data).unwrap();
        debug!("Sent.");
    }
}
