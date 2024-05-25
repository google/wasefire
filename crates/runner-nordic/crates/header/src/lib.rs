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

#[cfg(feature = "alloc")]
extern crate alloc;

use core::ptr::addr_of;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Header(u32);

impl Header {
    pub fn new(side: Side) -> Self {
        let addr = match side {
            Side::A => FIRMWARE_A,
            Side::B => FIRMWARE_B,
        };
        Header(addr)
    }

    pub fn timestamp(self) -> u32 {
        unsafe { read(self.0) }
    }

    pub fn attempt(self, index: u32) -> Attempt {
        assert!(index < 3);
        Attempt(self.0 + 4 * index)
    }

    pub fn has_firmware(self) -> bool {
        self.timestamp() != 0xffffffff
    }

    pub fn addr(self) -> u32 {
        self.0
    }

    pub fn side(self) -> Side {
        Side::new(self.addr()).unwrap()
    }

    pub fn firmware(&self) -> u32 {
        self.addr() + HEADER_LEN
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Side {
    A,
    B,
}

impl Side {
    pub fn current() -> Option<Self> {
        extern "C" {
            static mut __header_origin: u32;
        }
        Self::new(unsafe { addr_of!(__header_origin) } as u32)
    }

    fn new(addr: u32) -> Option<Self> {
        match addr {
            FIRMWARE_A => Some(Side::A),
            FIRMWARE_B => Some(Side::B),
            _ => None,
        }
    }
}

impl core::ops::Not for Side {
    type Output = Self;

    fn not(self) -> Self {
        match self {
            Side::A => Side::B,
            Side::B => Side::A,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Attempt(u32);

impl Attempt {
    pub fn free(self) -> bool {
        unsafe { read(self.0) == 0xffffffff }
    }

    pub fn addr(self) -> u32 {
        self.0
    }
}

unsafe fn read(addr: u32) -> u32 {
    unsafe { (addr as *const u32).read_volatile() }
}

// Keep those values in sync with the memory.x linker script.
const HEADER_LEN: u32 = 0x00000100;
const FIRMWARE_A: u32 = 0x00010000;
const FIRMWARE_B: u32 = 0x00080000;
