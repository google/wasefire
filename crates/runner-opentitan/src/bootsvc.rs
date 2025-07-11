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

use wasefire_common::platform::Side;
use wasefire_error::Error;

use crate::crypto::common::HashMode;
use crate::crypto::hash;

pub fn next_boot(next: Option<Side>, primary: Option<Side>) -> Result<(), Error> {
    let mut msg = [0u32; 64];
    msg[8] = u32::from_le_bytes(*b"BSVC");
    msg[9] = u32::from_le_bytes(*b"NEXT");
    msg[10] = 52;
    msg[11] = u32::from_le_bytes(*slot(next));
    msg[12] = u32::from_le_bytes(*slot(primary));
    let mut hash = hash::Context::init(HashMode::Sha256)?;
    hash.update(bytemuck::cast_slice(&msg[8 .. 13]))?;
    hash.finalize(&mut msg[.. 8])?;
    bytemuck::cast_slice_mut::<u32, u8>(&mut msg[.. 8]).reverse();
    for (addr, &word) in (0x40600008 ..).step_by(4).zip(msg.iter()) {
        unsafe { (addr as *mut u32).write_volatile(word) };
    }
    Ok(())
}

fn slot(side: Option<Side>) -> &'static [u8; 4] {
    match side {
        None => b"UUUU",
        Some(Side::A) => b"AA__",
        Some(Side::B) => b"__BB",
    }
}
