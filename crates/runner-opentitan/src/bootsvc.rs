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

pub fn next_boot(next: Option<Side>, primary: Option<Side>) {
    let mut msg = [0u8; 256];
    msg[32 ..][.. 4].copy_from_slice(b"BSVC");
    msg[36 ..][.. 4].copy_from_slice(b"NEXT");
    msg[40 ..][.. 4].copy_from_slice(&52u32.to_le_bytes());
    msg[44 ..][.. 4].copy_from_slice(slot(next));
    msg[48 ..][.. 4].copy_from_slice(slot(primary));
    let mut hash = crate::hmac::Hmac::start(None).unwrap();
    hash.update(&msg[32 .. 52]);
    hash.finalize((&mut msg[.. 32]).try_into().unwrap()).unwrap();
    msg[.. 32].reverse();
    for (addr, &word) in (0x40600008 ..).step_by(4).zip(msg.array_chunks()) {
        unsafe { (addr as *mut u32).write_volatile(u32::from_le_bytes(word)) };
    }
}

fn slot(side: Option<Side>) -> &'static [u8; 4] {
    match side {
        None => b"UUUU",
        Some(Side::A) => b"AA__",
        Some(Side::B) => b"__BB",
    }
}

#[cfg(feature = "debug")]
pub struct Format;

#[cfg(feature = "debug")]
impl defmt::Format for Format {
    fn format(&self, fmt: defmt::Formatter) {
        use alloc::vec::Vec;

        struct Ascii(u32);
        impl defmt::Format for Ascii {
            fn format(&self, fmt: defmt::Formatter) {
                match core::str::from_utf8(&self.0.to_le_bytes()) {
                    Ok(x) => defmt::write!(fmt, "{}", x),
                    Err(_) => defmt::write!(fmt, "0x{:08x}", self.0),
                }
            }
        }

        fn read_word(offset: u32) -> u32 {
            unsafe { ((0x40600000 + 4 * offset) as *const u32).read_volatile() }
        }
        fn read_words(mut offset: u32, mut length: usize) -> Vec<u8> {
            let mut result = Vec::new();
            while 0 < length {
                let word = read_word(offset);
                let len = core::cmp::min(length, 4);
                result.extend_from_slice(&word.to_le_bytes()[.. len]);
                offset += 1;
                length -= len;
            }
            result
        }

        defmt::write!(fmt, "boot_svc {} 0x{:x}", Ascii(read_word(0)), read_word(1));
        let length = read_word(12);
        if length < 32 || 224 < length {
            defmt::write!(fmt, " invalid length 0x{:08x}", length);
            return;
        }
        let msg = read_words(10, length as usize - 32);
        defmt::write!(fmt, " {} {}", Ascii(read_word(10)), Ascii(read_word(11)));
        for chunk in msg[12 ..].chunks(4) {
            defmt::write!(fmt, " ");
            chunk.iter().for_each(|x| defmt::write!(fmt, "{:02x}", x));
        }

        let mut hash = crate::hmac::Hmac::start(None).unwrap();
        hash.update(&msg);
        let mut actual = [0; 32];
        hash.finalize(&mut actual).unwrap();
        actual.reverse();
        let expected = read_words(2, 32);
        if actual[..] != expected[..] {
            defmt::write!(fmt, " invalid digest\n{=[u8]:02x}\n{=[u8]:02x}", actual, expected);
        }
    }
}
