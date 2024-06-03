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
//! different type parameter: `Api<Request>` and `Api<Response>` respectively. However, while the
//! host sends an `Api<Request>`, the device responds with an `ApiResult<T>` where `T` is the
//! service of the request.
//!
//! This high-level protocol is eventually wrapped in a lower-level protocol for a given transport,
//! for example USB. The host should provide enough time for the device to respond, but should
//! eventually move on in case no response will ever come, for example when the device is reset
//! before sending the response. Reciprocally, the device should accept a new request from the host
//! and cancel the request it was processing if any.

#![no_std]
#![feature(doc_auto_cfg)]
#![feature(macro_metavar_expr)]
#![feature(never_type)]

extern crate alloc;

use alloc::boxed::Box;

use wasefire_error::Error;
use wasefire_wire::Wire;
#[cfg(feature = "host")]
use wasefire_wire::Yoke;

pub mod applet;
pub mod platform;

/// Service description.
pub trait Service: 'static {
    /// Range of versions implementing this service.
    #[cfg(feature = "host")]
    const VERSIONS: Versions;
    type Request<'a>: Wire<'a>;
    type Response<'a>: Wire<'a>;
    #[cfg(feature = "host")]
    fn request(x: Self::Request<'_>) -> Api<Request>;
}

#[derive(Debug)]
pub enum Request {}
impl sealed::Direction for Request {
    type Type<'a, T: Service> = T::Request<'a>;
}

#[derive(Debug)]
#[cfg(feature = "_descriptor")]
pub enum Response {}
#[cfg(feature = "_descriptor")]
impl sealed::Direction for Response {
    type Type<'a, T: Service> = T::Response<'a>;
}

mod sealed {
    pub trait Direction: 'static {
        type Type<'a, T: crate::Service>: wasefire_wire::Wire<'a>;
    }
}

#[derive(Wire)]
#[wire(static = T, range = 2)]
pub enum ApiResult<'a, T: Service> {
    #[wire(tag = 0)]
    Ok(T::Response<'a>),
    #[wire(tag = 1)]
    Err(Error),
}

impl<'a> Api<'a, Request> {
    #[cfg(feature = "host")]
    pub fn encode(&self) -> Result<Box<[u8]>, Error> {
        wasefire_wire::encode(self)
    }

    #[cfg(feature = "device")]
    pub fn decode(data: &'a [u8]) -> Result<Self, Error> {
        wasefire_wire::decode(data)
    }
}

impl<'a, T: Service> ApiResult<'a, T> {
    #[cfg(feature = "device")]
    pub fn encode(&self) -> Result<Box<[u8]>, Error> {
        wasefire_wire::encode(self)
    }

    #[cfg(feature = "host")]
    pub fn decode(data: &[u8]) -> Result<ApiResult<T>, Error> {
        wasefire_wire::decode(data)
    }

    #[cfg(feature = "host")]
    pub fn decode_yoke(data: Box<[u8]>) -> Result<Yoke<ApiResult<'static, T>>, Error> {
        wasefire_wire::decode_yoke(data)
    }
}

#[derive(Debug, Copy, Clone, Wire)]
#[cfg(any(feature = "host", feature = "_descriptor"))]
pub struct Versions {
    pub min: u32,
    pub max: Option<u32>,
}

#[cfg(feature = "host")]
impl Versions {
    pub fn contains(&self, version: u32) -> Result<bool, Error> {
        match (self.min, self.max) {
            // Deprecated service. The device needs to be within the range.
            (min, Some(max)) => Ok((min ..= max).contains(&version)),
            // The device is too old.
            (min, None) if version < min => Ok(false),
            // The device is newer than the host, so we don't know if the service has been
            // deprecated for that device.
            (_, None) if VERSION < version => Err(Error::world(wasefire_error::Code::OutOfBounds)),
            // The device is older than the host but recent enough for the service.
            _ => Ok(true),
        }
    }
}

#[derive(Debug, Clone, Wire)]
#[cfg(feature = "_descriptor")]
pub struct Descriptor {
    pub tag: u32,
    pub versions: Versions,
}

#[cfg(feature = "_descriptor")]
impl core::fmt::Display for Descriptor {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let Descriptor { tag, versions: Versions { min, max } } = *self;
        match max {
            Some(max) => write!(f, "{tag} [{min} - {max}]"),
            None => write!(f, "{tag} [{min} -]"),
        }
    }
}

macro_rules! api {
    ($(#![$api:meta])* version = $version:literal; next = $next:literal; $(
        $(#[doc = $doc:literal])*
        $tag:literal [$min:literal - $($max:literal)?] $Name:ident: $request:ty => $response:ty
    ),*$(,)?) => {
        $(#[$api])* #[derive(Debug, Wire)]
        #[wire(static = T)]
        #[cfg_attr(feature = "host", wire(range = $next))]
        #[cfg_attr(not(feature = "_exhaustive"), non_exhaustive)]
        pub enum Api<'a, T: sealed::Direction> {
            $(
                $(#[doc = $doc])* $(#[cfg(feature = "host")] ${ignore($max)})? #[wire(tag = $tag)]
                $Name(T::Type<'a, $Name>),
            )*
        }
        $(
            $(#[doc = $doc])* $(#[cfg(feature = "host")] ${ignore($max)})? #[derive(Debug)]
            pub enum $Name {}
            $(#[cfg(feature = "host")] ${ignore($max)})?
            impl Service for $Name {
                #[cfg(feature = "host")]
                const VERSIONS: Versions = api!(versions $min $($max)?);
                type Request<'a> = $request;
                type Response<'a> = $response;
                #[cfg(feature = "host")]
                fn request(x: Self::Request<'_>) -> Api<Request> { Api::$Name(x) }
            }
        )*
        /// Device API version (or maximum supported device API version for host).
        pub const VERSION: u32 = $version;
        #[cfg(feature = "_descriptor")]
        pub const DESCRIPTORS: &'static [Descriptor] = &[
            $(
                $(#[cfg(feature = "host")] ${ignore($max)})?
                Descriptor { tag: $tag, versions: api!(versions $min $($max)?) },
            )*
        ];
    };

    (max) => (None);
    (max $max:literal) => (Some($max));

    (versions $min:literal $($max:literal)?) => (Versions { min: $min, max: api!(max $($max)?) });
}

api! {
    //! Protocol API parametric over the message direction.
    //!
    //! Variants gated by the `full` feature are deprecated. They won't be used by new devices.
    //! However, to support older devices, the host must be able to use them.
    version = 1;
    next = 6;

    /// Returns the device API version.
    0 [0 -] ApiVersion: () => u32,

    /// Sends a request to an applet.
    1 [0 -] AppletRequest: applet::Request<'a> => (),

    /// Reads a response from an applet.
    2 [0 -] AppletResponse: applet::AppletId => applet::Response<'a>,

    /// Reboots the platform.
    3 [0 -] PlatformReboot: () => !,

    /// Starts a direct tunnel with an applet.
    4 [0 -] AppletTunnel: applet::Tunnel<'a> => (),

    /// Returns platform information (e.g. serial and version).
    5 [1 -] PlatformInfo: () => platform::Info<'a>,
}
