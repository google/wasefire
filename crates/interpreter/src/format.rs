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

use alloc::vec::Vec;

pub fn leb128(wasm: &mut Vec<u8>, mut x: usize) {
    assert!(x <= u32::MAX as usize);
    while x > 127 {
        wasm.push(0x80 | (x & 0x7f) as u8);
        x >>= 7;
    }
    wasm.push(x as u8);
}

pub fn section(wasm: &mut Vec<u8>, id: u8, content: &[u8], name: &str) {
    assert!(id <= 12);
    wasm.push(id);
    let name_bytes = name.as_bytes();
    leb128(wasm, 1 + name_bytes.len() + content.len());
    leb128(wasm, name_bytes.len());
    wasm.extend_from_slice(name.as_bytes());
    wasm.extend_from_slice(content);
}
