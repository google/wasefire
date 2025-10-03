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

use wasefire_error::{Error, SpaceParam};

#[repr(u8)]
pub enum Space {
    Flash = 0x81,
    Hmac = 0x82,
    // 0xe0 and above are used by unwrap_status() below
}

impl From<Space> for u8 {
    fn from(value: Space) -> Self {
        value as u8
    }
}

impl SpaceParam for Space {}

pub fn unwrap_status(mut x: i32) -> Result<i32, Error> {
    if 0 <= x {
        return Ok(x);
    }
    x &= i32::MAX;
    let space = (x >> 26) as u8 | 0xe0;
    let code = ((x >> 16 << 5) | x & 0x1f) as u16 | 0x8000;
    Err(Error::new(space, code))
}

#[cfg(false)]
#[unsafe(no_mangle)]
extern "C" fn wasefire_debug_mark(mark: i32) {
    wasefire_logger::error!("DEBUG MARK {:08x} {}", mark, mark);
}
