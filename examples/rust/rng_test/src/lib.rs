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

//! Tests that the random number generator is working properly.

#![no_std]
wasefire::applet!();

fn main() {
    test_non_constant();
    debug!("End of tests.");
    scheduling::breakpoint();
}

fn test_non_constant() {
    debug!("test_non_constant(): This should generate 5 different buffers.");
    let mut buffers = [[0; 8]; 5];
    for buffer in buffers.iter_mut() {
        rng::fill_bytes(buffer).unwrap();
        debug!("- {buffer:02x?}");
    }
    for i in 1 .. buffers.len() {
        for j in 0 .. i {
            assert!(buffers[j] != buffers[i]);
        }
    }
}
