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
    assert!(platform::update::is_supported());
    debug!(" running    side: {}", platform::running_side());
    debug!(" running version: {:02x?}", platform::running_version());
    debug!("opposite version: {:02x?}", platform::opposite_version());
    let serial = usb::serial::UsbSerial;
    let mut length = [0; 4];
    serial::read_all(&serial, &mut length).unwrap();
    let mut length = usize::from_be_bytes(length);
    debug!("Start update of length {length}.");
    platform::update::initialize(false).unwrap();
    while 0 < length {
        const LENGTH: usize = 1024;
        let mut chunk = [0; LENGTH];
        let chunk = &mut chunk[.. core::cmp::min(LENGTH, length)];
        serial::read_all(&serial, chunk).unwrap();
        platform::update::process(chunk).unwrap();
        length -= chunk.len();
        debug!("Processed {} bytes ({length} remaining).", chunk.len());
    }
    platform::update::finalize().unwrap();
    unreachable!()
}
