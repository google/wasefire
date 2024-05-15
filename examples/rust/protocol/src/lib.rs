// Copyright 2024 Google LLC
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

//! Demonstrates simple RPC usage of the platform protocol.
//!
//! The applet converts the case of alphabetic ASCII characters from its request to its response.
//! Also switches I and O.

#![no_std]
wasefire::applet!();

use alloc::vec::Vec;

fn main() {
    let mut counter = 0;
    rpc::Listener::new(&platform::protocol::RpcProtocol, move |mut data: Vec<u8>| {
        data.iter_mut().for_each(switch_case);
        counter += 1;
        debug!("Converted {counter} lines.");
        data
    })
    .leak();
}

fn switch_case(x: &mut u8) {
    if x.is_ascii_alphabetic() {
        *x ^= 0x20;
    }
    if matches!(*x, b'I' | b'O' | b'i' | b'o') {
        *x ^= 0x6;
    }
}
