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

//! Implements some memory game.
//!
//! The applet combines is pure USB serial and describes its own usage when connecting on the USB
//! serial. It essentially shows a random base32 string (the length representing the level in the
//! game) for 3 seconds and gives 7 seconds to type it back.

#![no_std]
wasefire::applet!();

use alloc::rc::Rc;
use alloc::string::String;
use alloc::{format, vec};
use core::cell::Cell;
use core::time::Duration;

use wasefire::usb::serial::UsbSerial;

fn main() {
    let mut level = 3; // length of the string to remember
    let mut prompt = "Press ENTER when you are ready.";
    loop {
        serial::write_all(&UsbSerial, format!("\r\x1b[K{prompt}").as_bytes()).unwrap();

        // Make sure the player is ready.
        while serial::read_byte(&UsbSerial).unwrap() != 0x0d {}

        // Generate a question for this level.
        let mut question = vec![0; level];
        rng::fill_bytes(&mut question).unwrap();
        for byte in &mut question {
            const BASE32: [u8; 32] = *b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";
            *byte = BASE32[(*byte & 0x1f) as usize];
        }
        let mut question = String::from_utf8(question).unwrap();

        // Display the question.
        process(3, "Memorize this", &mut question, |_, x| x == 0x0d);

        // Read the answer.
        let mut answer = String::new();
        process(7, "Type what you remember", &mut answer, |answer, byte| {
            match byte {
                b'A' ..= b'Z' | b'2' ..= b'7' => answer.push(byte as char),
                b'a' ..= b'z' => answer.push(byte.to_ascii_uppercase() as char),
                0x7f => drop(answer.pop()),
                0x0d => return true,
                _ => (),
            }
            false
        });

        // Check the answer.
        if answer == question {
            level += 1;
            prompt = "\x1b[1;32mPromotion!\x1b[m Press ENTER for next level.";
        } else if level > 1 {
            level -= 1;
            prompt = "\x1b[1;31mDemotion...\x1b[m Press ENTER for previous level.";
        } else {
            prompt = "\x1b[1;41mRetry?\x1b[m Press ENTER to retry.";
        }
    }
}

fn process(
    max_secs: usize, prompt: &str, data: &mut String, update: impl Fn(&mut String, u8) -> bool,
) {
    let secs = Rc::new(Cell::new(0));
    let timer = timer::Timer::new({
        let time = secs.clone();
        move || time.set(time.get() + 1)
    });
    timer.start(timer::Periodic, Duration::from_secs(1));
    let mut done = false;
    while !done && secs.get() < max_secs {
        let secs = max_secs - secs.get();
        let message = format!("\r\x1b[K{prompt} ({secs} seconds remaining): \x1b[1m{data}\x1b[m");
        serial::write_all(&UsbSerial, message.as_bytes()).unwrap();
        let mut buffer = [0; 8];
        let reader = serial::Reader::new(&UsbSerial, &mut buffer);
        scheduling::wait_for_callback();
        let len = reader.result().unwrap();
        for &byte in &buffer[.. len] {
            done |= update(data, byte);
        }
    }
}
