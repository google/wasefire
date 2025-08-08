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

#[cfg(any(feature = "device", feature = "host"))]
use alloc::boxed::Box;

#[cfg(feature = "host")]
pub use connection::*;
use wasefire_error::Error;
use wasefire_wire::Wire;
#[cfg(feature = "host")]
use wasefire_wire::Yoke;

pub mod applet;
#[cfg(feature = "host")]
mod connection;
pub mod platform;
pub mod transfer;

/// Service description.
pub trait Service: 'static {
    #[cfg(feature = "host")]
    const NAME: &str;
    /// Range of versions implementing this service.
    #[cfg(feature = "host")]
    const VERSIONS: Versions;
    type Request<'a>: Wire<'a>;
    type Response<'a>: Wire<'a>;
    #[cfg(feature = "host")]
    fn request(x: Self::Request<'_>) -> Api<'_, Request>;
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

#[cfg(feature = "host")]
impl Api<'_, Request> {
    pub fn encode(&self) -> Result<Box<[u8]>, Error> {
        wasefire_wire::encode(self)
    }
}

#[cfg(feature = "device")]
impl<'a> Api<'a, Request> {
    pub fn decode(data: &'a [u8]) -> Result<Self, Error> {
        wasefire_wire::decode(data)
    }
}

impl<T: Service> ApiResult<'_, T> {
    #[cfg(feature = "device")]
    pub fn encode(&self) -> Result<Box<[u8]>, Error> {
        wasefire_wire::encode(self)
    }

    #[cfg(feature = "host")]
    pub fn decode(data: &[u8]) -> Result<ApiResult<'_, T>, Error> {
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
    ($(#![$api:meta])* $(
        $(#[doc = $doc:literal])*
        $tag:literal [$min:literal - $($max:literal)?] $Name:ident: $request:ty => $response:ty,
    )* next $next:literal [$version:literal - ]) => {
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
                const NAME: &str = stringify!($Name);
                #[cfg(feature = "host")]
                const VERSIONS: Versions = api!(versions $min $($max)?);
                type Request<'a> = $request;
                type Response<'a> = $response;
                #[cfg(feature = "host")]
                fn request(x: Self::Request<'_>) -> Api<'_, Request> { Api::$Name(x) }
            }
        )*
        /// Device API version (or maximum supported device API version for host).
        pub const VERSION: u32 = $version - 1;
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
    //! Deprecated variants are only available to the host (to support older devices).

    /// Returns the device API version.
    0 [0 -] ApiVersion: () => u32,

    /// Sends a request to an applet.
    1 [0 -] AppletRequest: applet::Request<'a> => (),

    /// Reads a response from an applet.
    2 [0 -] AppletResponse: applet::AppletId => Option<&'a [u8]>,

    /// Reboots the platform.
    3 [0 -] PlatformReboot: () => !,

    /// Starts a direct tunnel with an applet.
    4 [0 -] AppletTunnel: applet::Tunnel<'a> => (),

    /// (deprecated) Returns platform information (e.g. serial and version).
    ///
    /// This message is deprecated in favor of [`PlatformInfo`].
    5 [1 - 4] _PlatformInfo0: () => platform::_Info0<'a>,

    /// Calls a vendor-specific platform command.
    6 [2 -] PlatformVendor: &'a [u8] => &'a [u8],

    /// (deprecated) Returns the metadata for platform update.
    ///
    /// This message is deprecated in favor of [`PlatformVendor`], which is the message for
    /// vendor-specific messages.
    7 [3 - 4] _PlatformUpdateMetadata: () => &'a [u8],

    /// (deprecated) Updates the platform.
    ///
    /// This message is deprecated in favor of [`PlatformUpdate`].
    8 [3 - 6] _PlatformUpdate0: transfer::_Request0<'a> => (),

    /// (deprecated) Installs an applet.
    ///
    /// This message is deprecated in favor of [`AppletInstall`].
    9 [4 - 6] _AppletInstall0: transfer::_Request0<'a> => (),

    /// (deprecated) Uninstalls an applet.
    ///
    /// This message is deprecated in favor of [`AppletInstall`] with an empty transfer.
    10 [4 - 6] _AppletUninstall0: () => (),

    /// Returns the exit status of an applet, if not running.
    11 [4 -] AppletExitStatus: applet::AppletId => Option<applet::ExitStatus>,

    /// Locks a platform until reboot.
    ///
    /// This is useful for testing purposes by locking a platform before flashing a new one.
    12 [4 -] PlatformLock: () => (),

    /// Returns platform information (serial, version, running side, opposite version, etc).
    13 [5 -] PlatformInfo: () => platform::Info<'a>,

    /// Clears the store for the platform and all applets.
    ///
    /// The argument is the number of keys to protect. Using zero will clear all entries.
    14 [6 -] PlatformClearStore: usize => (),

    /// Updates the platform.
    15 [7 -] PlatformUpdate: transfer::Request<'a> => transfer::Response,

    /// Installs or uninstalls an applet.
    16 [7 -] AppletInstall: transfer::Request<'a> => transfer::Response,

    next 17 [8 -]
}
