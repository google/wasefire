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

//! Tests whether panic prints useful information on the debug output.
//!
//! The applet uses randomness to avoid the panic being optimized out, since the panic eventually
//! leads to the applet trapping.

#![no_std]
wasefire::applet!();

use core::slice;

use wasefire::debug::println;
use wasefire::rng::fill_bytes;

fn flip() -> bool {
    let mut x = 0u8;
    fill_bytes(slice::from_mut(&mut x)).unwrap();
    x & 1 == 1
}

fn main() {
    while flip() {
        println("not yet ready to die");
        continue;
    }
    panic!("ready to die");
}
