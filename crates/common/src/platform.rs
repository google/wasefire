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

//! Common platform-related items.

use wasefire_error::{Code, Error};

/// Platform side.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Side {
    A,
    B,
}

impl Side {
    /// Both sides in order.
    pub const LIST: [Side; 2] = [Side::A, Side::B];

    /// Returns the opposite side.
    pub fn opposite(self) -> Self {
        match self {
            Side::A => Side::B,
            Side::B => Side::A,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Side::A => "A",
            Side::B => "B",
        }
    }
}

impl From<bool> for Side {
    fn from(is_b: bool) -> Self {
        if is_b { Side::B } else { Side::A }
    }
}

impl core::fmt::Display for Side {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.name().fmt(f)
    }
}

impl core::str::FromStr for Side {
    type Err = Error;

    fn from_str(input: &str) -> Result<Self, Error> {
        match input {
            "A" => Ok(Side::A),
            "B" => Ok(Side::B),
            _ => Err(Error::user(Code::InvalidArgument)),
        }
    }
}
