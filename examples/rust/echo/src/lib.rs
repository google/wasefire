// Copyright 2022 Google LLC
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

//! Demonstrates simple USB serial usage.
//!
//! The applet simply echoes its input, except for alphabetic ASCII characters. Their case is
//! toggled first.

#![no_std]
wasefire::applet!();

fn main() {
    loop {
        let mut data = [0; 32];
        let len = usb::serial::read_any(&mut data).unwrap();
        let data = &mut data[.. len];
        data.iter_mut().for_each(switch_case);
        usb::serial::write_all(&data).unwrap();
    }
}

fn switch_case(x: &mut u8) {
    if x.is_ascii_alphabetic() {
        *x ^= 0x20;
    }
}
