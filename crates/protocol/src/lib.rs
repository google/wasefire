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

//! Wasefire platform protocol.
//!
//! This crate defines a high-level protocol between a host and a device. The host initiates
//! requests and the device responds. Requests and responses use the same [`Api`] but with a
//! different type parameter: `Api<Request>` and `Api<Response>` respectively.
//!
//! This high-level protocol is eventually wrapped in a lower-level protocol for a given transport,
//! for example USB. The host should provide enough time for the device to respond, but should
//! eventually move on in case no response will ever come, for example when the device is reset
//! before sending the response. Reciprocally, the device should accept a new request from the host
//! and cancel the request it was processing if any.

#![no_std]

extern crate alloc;

use alloc::boxed::Box;
use core::convert::Infallible;

use sealed::sealed;
use wasefire_error::{Code, Error};

pub use crate::reader::Reader;
pub use crate::writer::Writer;

mod reader;
pub mod service;
mod writer;

/// Zero-copy serialization.
#[sealed]
pub trait Serializable<'a>: Sized {
    /// Serializes without copy.
    fn serialize(&self, writer: &mut Writer<'a>);

    /// Deserializes without copy.
    fn deserialize(reader: &mut Reader<'a>) -> Result<Self, Error>;
}

/// Message direction between host and device.
#[sealed]
pub trait Direction<'a> {
    /// Returns the appropriate message type given a service.
    type Type<T: Service<'a>>: Serializable<'a>;
}

/// Requests from host to device.
pub enum Request {}
#[sealed]
impl<'a> Direction<'a> for Request {
    type Type<T: Service<'a>> = T::Request;
}

/// Responses from device to host.
#[derive(Debug)]
pub enum Response {}
#[sealed]
impl<'a> Direction<'a> for Response {
    type Type<T: Service<'a>> = T::Response;
}

/// Service description.
#[sealed]
pub trait Service<'a> {
    const IDENTIFIER: u8;
    type Request: Serializable<'a>;
    type Response: Serializable<'a>;
}

/// Protocol API parametric over the message direction.
#[derive(Debug)]
#[non_exhaustive]
pub enum Api<'a, T: Direction<'a>> {
    /// Errors reported by the device.
    ///
    /// This may be returned regardless of the request type.
    DeviceError(T::Type<service::DeviceError>),

    /// Sends a request to an applet.
    AppletRequest(T::Type<service::AppletRequest>),

    /// Reads a response from an applet.
    AppletResponse(T::Type<service::AppletResponse>),

    /// Reboots the platform.
    PlatformReboot(T::Type<service::PlatformReboot>),

    /// Starts a direct tunnel with an applet.
    AppletTunnel(T::Type<service::AppletTunnel>),
}

#[sealed]
impl<'a> Serializable<'a> for Infallible {
    fn serialize(&self, _: &mut Writer<'a>) {
        match *self {}
    }

    fn deserialize(_: &mut Reader<'a>) -> Result<Self, Error> {
        Err(Error::user(Code::InvalidArgument))
    }
}

#[sealed]
impl<'a> Serializable<'a> for () {
    fn serialize(&self, _: &mut Writer<'a>) {}

    fn deserialize(_: &mut Reader<'a>) -> Result<Self, Error> {
        Ok(())
    }
}

#[sealed]
impl<'a> Serializable<'a> for Error {
    fn serialize(&self, writer: &mut Writer) {
        writer.put_u8(self.space());
        writer.put_u16(self.code());
    }

    fn deserialize(reader: &mut Reader<'a>) -> Result<Self, Error> {
        let space = reader.get_u8()?;
        let code = reader.get_u16()?;
        Ok(Error::new(space, code))
    }
}

impl<'a, T: Direction<'a>> Api<'a, T> {
    pub fn serialize(&self) -> Box<[u8]> {
        let mut writer = Writer::new();
        macro_rules! serialize {
            ($N:ident, $x:ident) => {{
                writer.put_u8(<service::$N as Service>::IDENTIFIER);
                $x.serialize(&mut writer)
            }};
        }
        match self {
            Api::DeviceError(x) => serialize!(DeviceError, x),
            Api::AppletRequest(x) => serialize!(AppletRequest, x),
            Api::AppletResponse(x) => serialize!(AppletResponse, x),
            Api::PlatformReboot(x) => serialize!(PlatformReboot, x),
            Api::AppletTunnel(x) => serialize!(AppletTunnel, x),
        }
        writer.finalize()
    }

    pub fn deserialize(data: &'a [u8]) -> Result<Self, Error> {
        let mut reader = Reader::new(data);
        macro_rules! deserialize {
            ($N:ident) => {
                Api::$N(T::Type::<service::$N>::deserialize(&mut reader)?)
            };
        }
        let result = match reader.get_u8()? {
            service::DeviceError::IDENTIFIER => deserialize!(DeviceError),
            service::AppletRequest::IDENTIFIER => deserialize!(AppletRequest),
            service::AppletResponse::IDENTIFIER => deserialize!(AppletResponse),
            service::PlatformReboot::IDENTIFIER => deserialize!(PlatformReboot),
            service::AppletTunnel::IDENTIFIER => deserialize!(AppletTunnel),
            _ => return Err(Error::user(Code::InvalidArgument)),
        };
        reader.finalize()?;
        Ok(result)
    }
}
