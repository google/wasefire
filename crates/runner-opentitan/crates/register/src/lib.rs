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

//! Helpers for specifying register operations.

#![no_std]

use core::marker::PhantomData;

use bytemuck::TransparentWrapper;
use derive_where::derive_where;

/// Specifies a register.
pub trait RegSpec: Sized {
    /// The default value of the register.
    const DEFAULT: u32;

    /// Wrapper around a value read from the register.
    type Read: TransparentWrapper<RegRead<Self>>;

    /// Wrapper around a value to be written to the register.
    type Write: TransparentWrapper<RegWrite<Self>>;
}

/// Address of a specified register.
#[derive_where(Debug, Clone, Copy)]
pub struct RegAddr<Spec: RegSpec> {
    /// Address of the register.
    addr: u32,

    /// Specification of the register.
    spec: PhantomData<Spec>,
}

/// Values read from a register.
#[derive_where(Debug, Clone, Copy)]
pub struct RegRead<Spec: RegSpec> {
    /// Value read.
    value: u32,

    /// Specification of the register.
    spec: PhantomData<Spec>,
}

/// Values to be written to a register.
#[derive_where(Debug, Clone, Copy)]
pub struct RegWrite<Spec: RegSpec> {
    /// Address of the register.
    addr: u32,

    /// Value to be written.
    value: u32,

    /// Specification of the register.
    spec: PhantomData<Spec>,
}

impl<Spec: RegSpec> RegAddr<Spec> {
    /// Creates a register from its address.
    ///
    /// # Safety
    ///
    /// The address must point to a register specified by `Spec`. In particular, if `Spec: RegRead`
    /// then the address must be valid for read, and if `Spec: RegWrite` then it must be valid for
    /// write.
    #[inline]
    pub const unsafe fn new(addr: u32) -> Self {
        RegAddr { addr, spec: PhantomData }
    }

    /// Returns the address of the register.
    #[inline]
    pub fn addr(self) -> u32 {
        self.addr
    }

    /// Reads a register.
    #[inline]
    pub fn read(self) -> Spec::Read {
        Spec::Read::wrap(RegRead { value: self.read_raw(), spec: self.spec })
    }

    /// Returns the default value of a register, for write.
    #[inline]
    #[must_use]
    pub fn reset(self) -> Spec::Write {
        Spec::Write::wrap(RegWrite { addr: self.addr, value: Spec::DEFAULT, spec: self.spec })
    }

    /// Returns the current value of a register, for write.
    #[inline]
    #[must_use]
    pub fn modify(self) -> Spec::Write {
        let read = Spec::Read::peel(self.read());
        Spec::Write::wrap(RegWrite { addr: self.addr, value: read.value, spec: self.spec })
    }

    /// Reads a register as a u32.
    #[inline]
    pub fn read_raw(self) -> u32 {
        let ptr = self.addr as *const u32;
        // SAFETY: By `RegAddr::new()` requirement.
        unsafe { ptr.read_volatile() }
    }

    /// Writes a register as a u32.
    #[inline]
    pub fn write_raw(self, value: u32) {
        let ptr = self.addr as *mut u32;
        // SAFETY: By `RegAddr::new()` requirement.
        unsafe { ptr.write_volatile(value) };
    }

    /// Modifies a register as a u32.
    #[inline]
    pub fn modify_raw(self, op: impl FnOnce(u32) -> u32) {
        self.write_raw(op(self.read_raw()));
    }
}

impl<Spec: RegSpec> RegRead<Spec> {
    /// Returns the raw value.
    #[inline]
    pub fn value(self) -> u32 {
        self.value
    }

    /// Reads a field.
    #[inline]
    pub fn field(self, mask: u32) -> u32 {
        debug_assert!(valid_mask(mask));
        (self.value & mask) >> mask.trailing_zeros()
    }

    /// Reads a bit.
    #[inline]
    pub fn bit(self, bit: u8) -> bool {
        debug_assert!(valid_bit(bit));
        self.value & (1 << bit) != 0
    }
}

impl<Spec: RegSpec> RegWrite<Spec> {
    /// Returns the raw value.
    #[inline]
    pub fn value(self) -> u32 {
        self.value
    }

    /// Writes a field.
    #[inline]
    pub fn field(&mut self, mask: u32, value: u32) -> &mut Self {
        debug_assert!(valid_mask(mask));
        debug_assert_eq!(value & (mask >> mask.trailing_zeros()), value);
        self.value = (self.value & !mask) | (value << mask.trailing_zeros());
        self
    }

    /// Writes a bit.
    #[inline]
    pub fn bit(&mut self, bit: u8, value: bool) -> &mut Self {
        debug_assert!(valid_bit(bit));
        self.value = match value {
            true => self.value | (1 << bit),
            false => self.value & !(1 << bit),
        };
        self
    }

    /// Writes to the register.
    #[inline]
    pub fn write(self) {
        RegAddr { addr: self.addr, spec: self.spec }.write_raw(self.value);
    }
}

fn valid_mask(mut mask: u32) -> bool {
    mask >>= mask.trailing_zeros();
    mask != 0 && mask >> mask.trailing_ones() == 0
}

fn valid_bit(bit: u8) -> bool {
    bit < 32
}
