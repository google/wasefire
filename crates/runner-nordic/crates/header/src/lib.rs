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

#[repr(C)]
pub struct Header {
    pub timestamp: u32,
    pub attempt: [Attempt; 3],
}

#[repr(transparent)]
pub struct Attempt(u32);

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
        Self::new(unsafe { &__header_origin as *const u32 } as u32)
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

impl Attempt {
    pub const FREE: u32 = 0xffffffff;
    pub const USED: u32 = 0x00000000;

    pub fn addr(&self) -> u32 {
        self as *const _ as u32
    }

    pub fn free(&self) -> bool {
        self.0 == Self::FREE
    }

    pub fn used(&self) -> bool {
        !self.free()
    }
}

impl Header {
    pub fn new(side: Side) -> &'static Self {
        let addr = match side {
            Side::A => FIRMWARE_A,
            Side::B => FIRMWARE_B,
        };
        unsafe { &*(addr as *const Header) }
    }

    pub fn is_empty(&self) -> bool {
        self.timestamp == 0xffffffff
    }

    pub fn addr(&self) -> u32 {
        self as *const _ as u32
    }

    pub fn side(&self) -> Side {
        Side::new(self.addr()).unwrap()
    }

    pub fn firmware(&self) -> u32 {
        self.addr() + HEADER_LEN
    }
}

// Keep those values in sync with the memory.x linker script.
const HEADER_LEN: u32 = 0x00000100;
const FIRMWARE_A: u32 = 0x00010000;
const FIRMWARE_B: u32 = 0x00080000;
