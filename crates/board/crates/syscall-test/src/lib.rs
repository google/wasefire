// Copyright 2025 Google LLC
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

//! Board implementation for the syscall test.

#![no_std]

use wasefire_board_api::{AppletMemory, Error, Failure};

pub fn process(mut mem: impl AppletMemory, x2: u32, x3: u32, x4: u32) -> Result<u32, Failure> {
    match (x2, x3, x4) {
        (0, 0, x) => Ok(Error::decode(x as i32)?),
        (1, n, p) => Ok(mem.get(p, n)?.iter().map(|&x| x as u32).sum()),
        (2, n, p) => {
            let xs = mem.get_mut(p, n)?;
            (0 .. n).rev().zip(xs.iter_mut()).for_each(|(i, x)| *x = i as u8);
            Ok(n)
        }
        (3, n, 0) => {
            let p = mem.alloc(n, 1)?;
            mem.get_mut(p, n)?.iter_mut().for_each(|x| *x = n as u8);
            Ok(p)
        }
        _ => Err(Failure::TRAP),
    }
}
