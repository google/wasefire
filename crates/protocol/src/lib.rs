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
#![feature(doc_auto_cfg)]
#![feature(never_type)]

extern crate alloc;

use alloc::boxed::Box;

use wasefire_error::Error;
use wasefire_wire::Wire;

pub mod applet;
mod logic;

/// Message direction between host and device.
pub trait Direction: logic::Direction {}

/// Requests from host to device.
#[derive(Debug)]
pub enum Request {}
impl Direction for Request {}
impl logic::Direction for Request {
    type Type<'a, T: logic::Service> = T::Request<'a>;
}

/// Responses from device to host.
#[derive(Debug)]
pub enum Response {}
impl Direction for Response {}
impl logic::Direction for Response {
    type Type<'a, T: logic::Service> = T::Response<'a>;
}

impl<'a, T: Direction> Api<'a, T> {
    pub fn encode(&self) -> Result<Box<[u8]>, Error> {
        wasefire_wire::encode(self)
    }

    pub fn decode(data: &'a [u8]) -> Result<Self, Error> {
        wasefire_wire::decode(data)
    }
}

macro_rules! api {
    ($(#![$api:meta])* next = $next:literal; $(
        $(#[doc = $doc:literal])* $(#[cfg($cfg:meta)])*
        $tag:literal $Name:ident: $request:ty => $response:ty
    ),*$(,)?) => {
        $(#[$api])*
        #[derive(Debug, Wire)]
        #[wire(static = T)]
        #[cfg_attr(feature = "full", wire(range = $next))]
        #[non_exhaustive]
        pub enum Api<'a, T: Direction> {
            $(
                $(#[doc = $doc])* $(#[cfg($cfg)])* #[wire(tag = $tag)]
                $Name(T::Type<'a, $Name>),
            )*
        }
        $(
            $(#[cfg($cfg)])* #[derive(Debug)]
            pub enum $Name {}
            $(#[cfg($cfg)])*
            impl logic::Service for $Name {
                type Request<'a> = $request;
                type Response<'a> = $response;
            }
        )*
    };
}

api! {
    //! Protocol API parametric over the message direction.
    //!
    //! Variants gated by the `full` feature are deprecated. They won't be used by new devices.
    //! However, to support older devices, the host must be able to use them.
    next = 5;

    /// Errors reported by the device.
    ///
    /// This may be returned regardless of the request type.
    0 DeviceError: ! => Error,

    /// Sends a request to an applet.
    1 AppletRequest: applet::Request<'a> => (),

    /// Reads a response from an applet.
    2 AppletResponse: applet::AppletId => applet::Response<'a>,

    /// Reboots the platform.
    3 PlatformReboot: () => !,

    /// Starts a direct tunnel with an applet.
    4 AppletTunnel: applet::Tunnel<'a> => (),
}
