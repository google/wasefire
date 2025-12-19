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

use wasefire_common::platform::Side;
use wasefire_error::Error;
use wasefire_wire::{Wire, Yoke};

#[derive(Debug)]
pub enum DynInfo {
    V2(Yoke<Info<'static>>),
    V1(Yoke<_Info1<'static>>),
    V0(Yoke<_Info0<'static>>),
}

#[cfg(feature = "host")]
impl DynInfo {
    pub fn serial(&self) -> &[u8] {
        match self {
            DynInfo::V2(x) => &x.get().serial,
            DynInfo::V1(x) => &x.get().serial,
            DynInfo::V0(x) => &x.get().serial,
        }
    }

    pub fn applet_kind(&self) -> Option<AppletKind> {
        match self {
            DynInfo::V2(x) => Some(x.get().applet_kind),
            DynInfo::V1(_) => None,
            DynInfo::V0(_) => None,
        }
    }

    pub fn running_side(&self) -> Option<Side> {
        match self {
            DynInfo::V2(x) => Some(x.get().running_side),
            DynInfo::V1(x) => Some(x.get().running_side),
            DynInfo::V0(_) => None,
        }
    }

    pub fn running_version(&self) -> &[u8] {
        match self {
            DynInfo::V2(x) => &x.get().running_version,
            DynInfo::V1(x) => &x.get().running_version,
            DynInfo::V0(x) => &x.get().version,
        }
    }

    pub fn opposite_version(&self) -> Option<Result<&[u8], Error>> {
        let opposite = match self {
            DynInfo::V2(x) => &x.get().opposite_version,
            DynInfo::V1(x) => &x.get().opposite_version,
            DynInfo::V0(_) => return None,
        };
        Some(match opposite {
            Ok(x) => Ok(x),
            Err(e) => Err(*e),
        })
    }
}

#[derive(Debug, Wire)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Info<'a> {
    pub serial: Cow<'a, [u8]>,
    pub applet_kind: AppletKind,
    pub running_side: Side,
    pub running_version: Cow<'a, [u8]>,
    pub opposite_version: Result<Cow<'a, [u8]>, Error>,
}

#[derive(Debug, Wire)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct _Info1<'a> {
    pub serial: Cow<'a, [u8]>,
    pub running_side: Side,
    pub running_version: Cow<'a, [u8]>,
    pub opposite_version: Result<Cow<'a, [u8]>, Error>,
}

#[derive(Debug, Wire)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct _Info0<'a> {
    pub serial: Cow<'a, [u8]>,
    pub version: Cow<'a, [u8]>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Wire)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum AppletKind {
    Wasm,
    Pulley,
    Native,
}

impl AppletKind {
    pub fn name(&self) -> &'static str {
        match self {
            AppletKind::Wasm => "wasm",
            AppletKind::Pulley => "pulley",
            AppletKind::Native => "native",
        }
    }
}

impl core::fmt::Display for AppletKind {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.name().fmt(f)
    }
}
