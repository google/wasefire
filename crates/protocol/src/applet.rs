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

use alloc::borrow::Cow;

use wasefire_error::{Code, Error};
use wasefire_wire::Wire;

use crate::common::{Hexa, Name};

#[derive(Debug, Default, Clone, Wire)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Metadata<'a> {
    /// Name of the applet.
    pub name: Name<'a>,

    /// Version of the applet.
    pub version: Hexa<'a>,
}

impl<'a> Metadata<'a> {
    /// Extracts the metadata suffix from an applet payload.
    ///
    /// The payload is the concatenation of:
    /// - The applet (N bytes). After the call, the payload will be the applet.
    /// - The metadata.
    /// - The value N encoded as 4 bytes in big-endian.
    pub fn new(payload: &mut &'a [u8]) -> Result<Metadata<'a>, Error> {
        let (data, len) =
            payload.split_last_chunk::<4>().ok_or(Error::user(Code::InvalidLength))?;
        let len = u32::from_be_bytes(*len) as usize;
        let (applet, metadata) =
            data.split_at_checked(len).ok_or(Error::user(Code::InvalidState))?;
        *payload = applet;
        wasefire_wire::decode(metadata)
    }
}

#[derive(Debug, Wire)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Request<'a> {
    pub applet_id: AppletId,
    pub request: Cow<'a, [u8]>,
}

#[derive(Debug, Copy, Clone, Wire)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AppletId;

#[derive(Debug, Wire)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Tunnel<'a> {
    pub applet_id: AppletId,
    pub delimiter: Cow<'a, [u8]>,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Wire)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ExitStatus {
    /// The applet exited successfully.
    Exit,

    /// The applet aborted (e.g. it panicked).
    Abort,

    /// The applet trapped (e.g. bad memory access).
    Trap,

    /// The applet was killed (e.g. it was uninstalled).
    Kill,
}

#[cfg(feature = "host")]
impl core::fmt::Display for ExitStatus {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ExitStatus::Exit => write!(f, "The applet exited"),
            ExitStatus::Abort => write!(f, "The applet aborted"),
            ExitStatus::Trap => write!(f, "The applet trapped"),
            ExitStatus::Kill => write!(f, "The applet was killed"),
        }
    }
}
