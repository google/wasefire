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

#![no_std]
wasefire::applet!();

use alloc::boxed::Box;
use core::cell::Cell;

fn main() {
    let byte = make();
    usb::serial::write_all(&[byte.get()]).unwrap();
}

fn make() -> Box<Cell<u8>> {
    Box::new(Cell::new(18))
}

#[cfg(test)]
mod tests {
    use wasefire_stub as _;

    use super::*;

    #[test]
    fn test_make() {
        // We want to make sure types in core and alloc also work in tests.
        assert_eq!(make().get(), 18);
    }

    #[test]
    fn test_sha256() {
        assert_eq!(
            crypto::hash::sha256(b"hello"),
            Ok([
                0x2c, 0xf2, 0x4d, 0xba, 0x5f, 0xb0, 0xa3, 0x0e, 0x26, 0xe8, 0x3b, 0x2a, 0xc5, 0xb9,
                0xe2, 0x9e, 0x1b, 0x16, 0x1e, 0x5c, 0x1f, 0xa7, 0x42, 0x5e, 0x73, 0x04, 0x33, 0x62,
                0x93, 0x8b, 0x98, 0x24,
            ])
        );
    }
}
