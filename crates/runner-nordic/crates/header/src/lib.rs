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

use wasefire_common::addr_of_symbol;
use wasefire_common::platform::Side;
use wasefire_one_of::exactly_one_of;

exactly_one_of!["board-devkit", "board-dongle"];

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Header(u32);

// Format (packed):
// 0x00 version (4 bytes)
// 0x04 attempts (3 times 4 bytes)
// 0x10 name (24 bytes)
// 0x28
impl Header {
    pub fn new(side: Side) -> Self {
        let addr = match side {
            Side::A => FIRMWARE_A,
            Side::B => FIRMWARE_B,
        };
        Header(addr)
    }

    pub fn version(self) -> u32 {
        unsafe { read(self.0 + 0x00) }
    }

    pub fn attempt(self, index: u32) -> Attempt {
        assert!(index < 3);
        Attempt(self.0 + 0x04 + index * 4)
    }

    pub fn name(self) -> [u8; 24] {
        unsafe { read(self.0 + 0x10) }
    }

    pub fn has_firmware(self) -> bool {
        self.version() != 0xffffffff
    }

    pub fn addr(self) -> u32 {
        self.0
    }

    pub fn side(self) -> Side {
        new_side(self.addr()).unwrap()
    }

    pub fn firmware(&self) -> u32 {
        self.addr() + HEADER_LEN
    }
}

pub fn running_side() -> Option<Side> {
    new_side(addr_of_symbol!(__header_origin) as u32)
}

fn new_side(addr: u32) -> Option<Side> {
    match addr {
        FIRMWARE_A => Some(Side::A),
        FIRMWARE_B => Some(Side::B),
        _ => None,
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Attempt(u32);

impl Attempt {
    pub fn free(self) -> bool {
        unsafe { read::<u32>(self.0) == 0xffffffff }
    }

    pub fn addr(self) -> u32 {
        self.0
    }
}

unsafe fn read<T>(addr: u32) -> T {
    unsafe { (addr as *const T).read_volatile() }
}

const HEADER_LEN: u32 = 0x00000100;

#[cfg(feature = "board-devkit")]
const FIRMWARE_A: u32 = 0x00010000;
#[cfg(feature = "board-devkit")]
const FIRMWARE_B: u32 = 0x00064000;

#[cfg(feature = "board-dongle")]
const FIRMWARE_A: u32 = 0x00008000;
#[cfg(feature = "board-dongle")]
const FIRMWARE_B: u32 = 0x00050000;

#[cfg(feature = "board-dongle")]
pub const STORE_RANGE: core::ops::Range<u32> = 0x000da000 .. 0x000e0000;
