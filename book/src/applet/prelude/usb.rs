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

//{ ANCHOR: all
#![no_std]
wasefire::applet!();

use alloc::rc::Rc;
use alloc::string::String;
use alloc::{format, vec};
use core::cell::Cell;
use core::time::Duration;

fn main() {
    //{ ANCHOR: state
    let mut level = 3; // length of the string to remember
    let mut prompt = "Press ENTER when you are ready.";
    //} ANCHOR_END: state
    //{ ANCHOR: loop
    loop {
    //} ANCHOR_END: loop
        //{ ANCHOR: prompt
        usb::serial::write_all(format!("\r\x1b[K{prompt}").as_bytes()).unwrap();
        //} ANCHOR_END: prompt

        //{ ANCHOR: ready
        // Make sure the player is ready.
        while usb::serial::read_byte().unwrap() != 0x0d {}
        //} ANCHOR_END: ready

        //{ ANCHOR: generate
        // Generate a question for this level.
        let mut question = vec![0; level];
        rng::fill_bytes(&mut question).unwrap();
        for byte in &mut question {
            const BASE32: [u8; 32] = *b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";
            *byte = BASE32[(*byte & 0x1f) as usize];
        }
        let mut question = String::from_utf8(question).unwrap();
        //} ANCHOR_END: generate

        //{ ANCHOR: question
        // Display the question.
        process(3, "Memorize this", &mut question, |_, x| x == 0x0d);
        //} ANCHOR_END: question

        //{ ANCHOR: answer
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
        //} ANCHOR_END: answer

        //{ ANCHOR: update
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
        //} ANCHOR_END: update
    }
}

//{ ANCHOR: process
fn process(
    max_secs: usize, prompt: &str, data: &mut String, update: impl Fn(&mut String, u8) -> bool,
) {
//} ANCHOR_END: process
    //{ ANCHOR: process_setup
    let secs = Rc::new(Cell::new(0));
    let timer = clock::Timer::new({
        let time = secs.clone();
        move || time.set(time.get() + 1)
    });
    timer.start(clock::Periodic, Duration::from_secs(1));
    //} ANCHOR_END: process_setup
    //{ ANCHOR: process_loop
    let mut done = false;
    while !done && secs.get() < max_secs {
    //} ANCHOR_END: process_loop
        //{ ANCHOR: process_write
        let secs = max_secs - secs.get();
        let message = format!("\r\x1b[K{prompt} ({secs} seconds remaining): \x1b[1m{data}\x1b[m");
        usb::serial::write_all(message.as_bytes()).unwrap();
        //} ANCHOR_END: process_write
        //{ ANCHOR: process_reader
        let mut buffer = [0; 8];
        let reader = usb::serial::Reader::new(&mut buffer);
        //} ANCHOR_END: process_reader
        //{ ANCHOR: process_wait
        scheduling::wait_for_callback();
        //} ANCHOR_END: process_wait
        //{ ANCHOR: process_read
        let len = reader.result().unwrap();
        for &byte in &buffer[.. len] {
            done |= update(data, byte);
        }
        //} ANCHOR_END: process_read
    }
}
//} ANCHOR_END: all
