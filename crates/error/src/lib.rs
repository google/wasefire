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

//! Applet and board API errors.

#![no_std]

use num_enum::{IntoPrimitive, TryFromPrimitive, TryFromPrimitiveError};

/// API errors.
///
/// Errors are equivalent to `u32` but only using the 24 least significant bits: 8 bits for the
/// error space and 16 bits for the error code. The 8 most significant bits are zero. It is possible
/// to encode a `Result<u32, Error>` into a `i32` as long as the success value only uses the 31
/// least significant bits. Non-negative values encode success, while negative values encode the
/// error by taking its bitwise complement (thus setting the 8 most significant bits to 1).
#[derive(Default, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[repr(transparent)]
pub struct Error(u32);

impl Error {
    /// Creates a new error.
    pub fn new(space: impl SpaceParam, code: impl CodeParam) -> Self {
        Error((space.into() as u32) << 16 | code.into() as u32)
    }

    /// Returns the error space.
    pub fn space(self) -> u8 {
        (self.0 >> 16) as u8
    }

    /// Returns the error code.
    pub fn code(self) -> u16 {
        self.0 as u16
    }

    /// Creates a user error.
    pub fn user(code: impl CodeParam) -> Self {
        Error::new(Space::User, code)
    }

    /// Creates an internal error.
    pub fn internal(code: impl CodeParam) -> Self {
        Error::new(Space::Internal, code)
    }

    /// Creates a world error.
    pub fn world(code: impl CodeParam) -> Self {
        Error::new(Space::World, code)
    }

    /// Decodes a signed integer as a result (where errors are negative values).
    ///
    /// # Panics
    ///
    /// Panics if the signed integer is smaller than -16777216 (0xff000000).
    pub fn decode(result: i32) -> Result<u32, Self> {
        if result < 0 {
            let error = !result as u32;
            assert!(error & 0xff000000 == 0);
            Err(Error(error))
        } else {
            Ok(result as u32)
        }
    }

    /// Encodes a result (where errors are negative values) as a signed integer.
    ///
    /// # Panics
    ///
    /// Panics if the result is a success greater than 2147483647 (0x7fffffff).
    pub fn encode(result: Result<u32, Self>) -> i32 {
        match result {
            Ok(value) => {
                let value = value as i32;
                assert!(0 <= value);
                value
            }
            Err(Error(error)) => !error as i32,
        }
    }
}

pub trait SpaceParam: Into<u8> {}
impl SpaceParam for Space {}
impl SpaceParam for u8 {}

pub trait CodeParam: Into<u16> {}
impl CodeParam for Code {}
impl CodeParam for u16 {}

/// Common error spaces.
///
/// Values from 0 to 127 (0x7f) are reserved for common error spaces and defined by this enum.
/// Values from 128 (0x80) to 255 (0xff) are reserved for implementation-specific error spaces.
#[derive(Debug, Copy, Clone, TryFromPrimitive, IntoPrimitive)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
#[repr(u8)]
pub enum Space {
    Generic = 0,

    /// The user made an error.
    User = 1,

    /// The implementation failed.
    Internal = 2,

    /// The world returned an error.
    World = 3,
}

/// Common error codes.
///
/// Values from 0 to 32767 (0x7fff) are reserved for common error codes and defined by this enum.
/// Values from 32768 (0x8000) to 65535 (0xffff) are reserved for implementation-specific error
/// codes.
#[derive(Debug, Copy, Clone, TryFromPrimitive, IntoPrimitive)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[non_exhaustive]
#[repr(u16)]
pub enum Code {
    Generic = 0,
    NotImplemented = 1,
    NotFound = 2,
    BadSize = 3,
    BadAlign = 4,
    NoPermission = 5,
    NotEnough = 6,
    BadState = 7,
}

impl core::fmt::Debug for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{self}")
    }
}

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        // Keep in sync with defmt::Format.
        match Space::try_from_primitive(self.space()) {
            Ok(x) => write!(f, "{x:?}")?,
            Err(TryFromPrimitiveError { number: x }) => write!(f, "[{x:02x}]")?,
        }
        write!(f, ":")?;
        match Code::try_from_primitive(self.code()) {
            Ok(x) => write!(f, "{x:?}"),
            Err(TryFromPrimitiveError { number: x }) => write!(f, "[{x:04x}]"),
        }
    }
}

#[cfg(feature = "defmt")]
impl defmt::Format for Error {
    fn format(&self, fmt: defmt::Formatter) {
        // Keep in sync with core::fmt::Display.
        use defmt::write;
        match Space::try_from_primitive(self.space()) {
            Ok(x) => write!(fmt, "{:?}", x),
            Err(TryFromPrimitiveError { number: x }) => write!(fmt, "[{:02x}]", x),
        }
        write!(fmt, ":");
        match Code::try_from_primitive(self.code()) {
            Ok(x) => write!(fmt, "{:?}", x),
            Err(TryFromPrimitiveError { number: x }) => write!(fmt, "[{:04x}]", x),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_ok() {
        assert_eq!(Error::new(Space::User, Code::BadSize), Error(0x010003));
        assert_eq!(Error::new(0xab, 0x1234u16), Error(0xab1234));
    }
}
