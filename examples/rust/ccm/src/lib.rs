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

//! Demonstrates simple AES-CCM usage.
//!
//! The applet also uses USB serial and RNG. The applet starts listening on the USB serial. A random
//! key and IV are generated before waiting for the next character. If the character is not a digit,
//! it is ignored (but refreshes the key and IV). Digits are mapped to their own value except `0`
//! which is mapped to 10. When a digit is received, a random value of the associated length is
//! created. That value is first encrypted then decrypted.

#![no_std]
wasefire::applet!();

use alloc::{format, vec};

use wasefire::crypto::ccm;
use wasefire::rng::fill_bytes;
use wasefire::usb::serial::{read_byte, write_all};

fn main() {
    loop {
        let mut key = [0; 16];
        fill_bytes(&mut key);
        write_hex("key", &key);
        let mut iv = [0; 8];
        fill_bytes(&mut iv);
        write_hex("iv", &iv);

        let len = match read_byte().unwrap() {
            c @ b'1' ..= b'9' => c - b'0',
            b'0' => 10,
            _ => continue,
        } as usize;
        let mut clear = vec![0; len];
        fill_bytes(&mut clear);
        write_hex("clear", &clear);

        let cipher = ccm::encrypt(&key, &iv, &clear).unwrap();
        write_hex("cipher", &cipher);

        let result = ccm::decrypt(&key, &iv, &cipher).unwrap();
        write_hex("result", &result);

        writeln("");
    }
}

fn write_hex(n: &str, xs: &[u8]) {
    writeln(&format!("{n}: {xs:02x?}"));
}

fn writeln(m: &str) {
    write_all(format!("{m}\r\n").as_bytes()).unwrap();
}
