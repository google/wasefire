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

use wasefire_common::platform::Side;
use wasefire_error::Error;
use wasefire_wire::Wire;

#[derive(Debug, Wire)]
pub struct Info<'a> {
    pub serial: &'a [u8],
    pub running_side: Side,
    pub running_version: &'a [u8],
    pub opposite_version: Result<&'a [u8], Error>,
}

#[cfg(feature = "host")]
impl core::fmt::Display for Info<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        use data_encoding::HEXLOWER as HEX;
        writeln!(f, "  serial: {}", HEX.encode_display(self.serial))?;
        writeln!(f, "    side: {}", self.running_side)?;
        writeln!(f, " version: {}", HEX.encode_display(self.running_version))?;
        writeln!(f, "opposite: {}", match self.opposite_version {
            Ok(x) => HEX.encode(x),
            Err(e) => alloc::format!("{e}"),
        })
    }
}

#[derive(Debug, Wire)]
pub struct Info0<'a> {
    pub serial: &'a [u8],
    pub version: &'a [u8],
}
