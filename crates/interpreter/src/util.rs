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

use alloc::vec::Vec;

// TODO(dev/fast-interp): Add debug asserts when `off` is positive and negative, and `toctou`
// support.
pub fn offset_front<T>(cur: &[T], off: isize) -> &[T] {
    unsafe {
        core::slice::from_raw_parts(cur.as_ptr().offset(off), (cur.len() as isize - off) as usize)
    }
}

pub fn leb128(mut x: usize, wasm: &mut Vec<u8>) {
    assert!(x <= u32::MAX as usize);
    while x > 127 {
        wasm.push(0x80 | (x & 0x7f) as u8);
        x >>= 7;
    }
    wasm.push(x as u8);
}
