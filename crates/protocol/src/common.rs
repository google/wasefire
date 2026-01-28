// Copyright 2026 Google LLC
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
use core::ops::Deref;

use wasefire_error::{Code, Error};
use wasefire_wire::Wire;

/// String made of ASCII graphic characters only.
#[derive(Debug, Default, Wire)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[wire(refine = Name::new)]
pub struct Name<'a>(Cow<'a, str>);

impl<'a> Deref for Name<'a> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> Name<'a> {
    pub fn new(data: Cow<'a, str>) -> Result<Self, Error> {
        if data.bytes().all(|x| x.is_ascii_graphic()) {
            Ok(Self(data))
        } else {
            Err(Error::user(Code::InvalidArgument))
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Wire)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum AppletKind {
    Wasm,
    Pulley,
    Native,
}

#[cfg(feature = "host")]
impl AppletKind {
    pub fn name(&self) -> &'static str {
        match self {
            AppletKind::Wasm => "wasm",
            AppletKind::Pulley => "pulley",
            AppletKind::Native => "native",
        }
    }
}

#[cfg(feature = "host")]
impl core::fmt::Display for AppletKind {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.name().fmt(f)
    }
}
