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

//! Demonstrates simple RNG usage.
//!
//! The applet also uses USB serial. The applet starts listening for digits on the USB serial. Any
//! other character is ignored. Digits are mapped to their own value, except `0` which maps to 10.
//! When a digit is received, a random values of the associated length is generated.

#![no_std]
wasefire::applet!();

use alloc::{format, vec};

use wasefire::usb::serial::UsbSerial;

fn main() {
    loop {
        let len = match serial::read_byte(&UsbSerial).unwrap() {
            c @ b'1' ..= b'9' => c - b'0',
            b'0' => 10,
            _ => continue,
        };
        let mut buf = vec![0; len as usize];
        rng::fill_bytes(&mut buf).unwrap();
        serial::write_all(&UsbSerial, format!("{buf:02x?}\r\n").as_bytes()).unwrap();
    }
}
