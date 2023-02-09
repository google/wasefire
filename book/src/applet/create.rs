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

//{ ANCHOR: all
//{ ANCHOR: nostd
#![no_std]
//} ANCHOR_END: nostd

//{ ANCHOR: alloc
extern crate alloc;
//} ANCHOR_END: alloc

//{ ANCHOR: prelude
use prelude::*;
//} ANCHOR_END: prelude

//{ ANCHOR: nomangle
#[no_mangle]
//} ANCHOR_END: nomangle
//{ ANCHOR: externc
pub extern "C" fn main() {
//} ANCHOR_END: externc
    println!("hello world");
}
//} ANCHOR_END: all
