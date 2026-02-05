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
use alloc::vec::Vec;

use wasefire_error::{Code, Error};
use wasefire_wire::Wire;

use crate::common::AppletKind;
use crate::{Service as _, applet, platform};

/// Bundle magic header.
///
/// This is meant to read as Wasefire in hexadecimal.
pub const MAGIC: [u8; 4] = [0x3a, 0x5e, 0xf1, 0x2e];

/// Bundle file extension.
///
/// This is meant to read as Wasefire File Bundle.
pub const EXTENSION: &str = "wfb";

/// Bundle content.
///
/// The version and type is encoded by the enum tag.
#[derive(Debug, Wire)]
#[wire(range = 2)]
pub enum Bundle<'a> {
    #[wire(tag = 0)]
    Platform0(Platform0<'a>),
    #[wire(tag = 1)]
    Applet0(Applet0<'a>),
}

impl<'a> Bundle<'a> {
    /// Encodes the bundle to its associated file content.
    pub fn encode(&self) -> Result<Vec<u8>, Error> {
        let mut content = MAGIC.to_vec();
        wasefire_wire::encode_suffix(&mut content, self)?;
        Ok(content)
    }

    /// Decodes a bundle from its associated file content.
    pub fn decode(content: &'a [u8]) -> Result<Self, Error> {
        let Some((&magic, content)) = content.split_first_chunk::<4>() else {
            return Err(Error::user(Code::InvalidLength));
        };
        if magic != MAGIC {
            return Err(Error::user(Code::InvalidState));
        }
        wasefire_wire::decode(content)
    }

    /// Assumes the bundle is for a platform.
    pub fn platform(self) -> Result<Platform<'a>, Error> {
        match self {
            Bundle::Platform0(x) => Ok(Platform::V0(x)),
            _ => Err(Error::user(Code::InvalidArgument)),
        }
    }

    /// Assumes the bundle is for an applet.
    pub fn applet(self) -> Result<Applet<'a>, Error> {
        match self {
            Bundle::Applet0(x) => Ok(Applet::V0(x)),
            _ => Err(Error::user(Code::InvalidArgument)),
        }
    }
}

/// Platform update.
#[derive(Debug, Wire)]
pub struct Platform0<'a> {
    /// Metadata about the platform.
    ///
    /// This metadata is redundant with the platform data for both sides, but contrary to the
    /// platform data, this metadata can be accessed by generic tools (without knowning the
    /// platform-specific data format).
    pub metadata: platform::SideInfo0<'a>,

    /// The platform data when stored on the A side.
    pub side_a: Cow<'a, [u8]>,

    /// The platform data when stored on the B side.
    ///
    /// This can be empty if the platform has only one side.
    pub side_b: Cow<'a, [u8]>,
}

pub enum Platform<'a> {
    V0(Platform0<'a>),
}

impl<'a> Platform<'a> {
    /// Returns the device-specific payloads to update the platform.
    pub fn payloads(&self) -> (Vec<u8>, Vec<u8>) {
        (self.side_a().to_vec(), self.side_b().to_vec())
    }

    /// Returns the side A of the platform.
    pub fn side_a(&self) -> &[u8] {
        match self {
            Platform::V0(x) => &x.side_a,
        }
    }

    /// Returns the side B of the platform.
    pub fn side_b(&self) -> &[u8] {
        match self {
            Platform::V0(x) => &x.side_b,
        }
    }
}

/// Applet install (and update).
#[derive(Debug, Wire)]
pub struct Applet0<'a> {
    /// The applet kind.
    ///
    /// This is redundant with the applet data at this time (pulley and wasm), but provided for
    /// convenience.
    pub kind: AppletKind,

    /// The applet metadata.
    ///
    /// The CLI or Web UI will suffix this metadata to the applet data in the format supported
    /// by the device.
    pub metadata: applet::Metadata0<'a>,

    /// The applet data.
    pub data: Cow<'a, [u8]>,
}

pub enum Applet<'a> {
    V0(Applet0<'a>),
}

impl<'a> Applet<'a> {
    /// Returns the device-specific payload to install the applet.
    ///
    /// The version is the protocol version of the device.
    pub fn payload(&self, version: u32) -> Result<Vec<u8>, Error> {
        let mut result = self.data().to_vec();
        let len = result.len() as u32;
        match self {
            Applet::V0(x) => {
                // TODO: We should convert the metadata.
                if !crate::AppletMetadata0::VERSIONS.contains(version)? {
                    return Err(Error::internal(Code::NotImplemented));
                }
                wasefire_wire::encode_suffix(&mut result, &x.metadata)?;
            }
        }
        result.extend_from_slice(&len.to_be_bytes());
        Ok(result)
    }

    /// Returns the data of the applet.
    pub fn data(&self) -> &[u8] {
        match self {
            Applet::V0(x) => &x.data,
        }
    }
}
