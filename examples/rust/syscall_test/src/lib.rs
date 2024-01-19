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

//! Tests that the board-specific syscall API is working properly.

#![no_std]
wasefire::applet!();

fn main() {
    test_error();
    debug::exit(true);
}

fn test_error() {
    debug!("test_error(): Check that errors are correctly converted.");
    for (x, r) in [
        (0, Ok(0)),
        (0x7fffffff, Ok(0x7fffffff)),
        (0xff000000, Err(Error::new(0xff, 0xffff))),
        (0xfffefffd, Err(Error::user(2))),
        (0xffffffff, Err(Error::default())),
    ] {
        debug!("- {x:08x} -> {r:?}");
        debug::assert_eq(&syscall(0, 0, 0, x), &r);
    }
}
