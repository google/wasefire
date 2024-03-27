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

use core::convert::Infallible;

use sealed::sealed;
use wasefire_error::Error;

pub mod applet;

#[derive(Debug)]
pub enum DeviceError {}
#[sealed]
impl<'a> crate::Service<'a> for DeviceError {
    const IDENTIFIER: u8 = 0x00;
    type Request = Infallible;
    type Response = Error;
}

#[derive(Debug)]
pub enum AppletRequest {}
#[sealed]
impl<'a> crate::Service<'a> for AppletRequest {
    const IDENTIFIER: u8 = 0x01;
    type Request = applet::Request<'a>;
    type Response = ();
}

#[derive(Debug)]
pub enum AppletResponse {}
#[sealed]
impl<'a> crate::Service<'a> for AppletResponse {
    const IDENTIFIER: u8 = 0x02;
    type Request = applet::AppletId;
    type Response = applet::Response<'a>;
}

#[derive(Debug)]
pub enum PlatformReboot {}
#[sealed]
impl<'a> crate::Service<'a> for PlatformReboot {
    const IDENTIFIER: u8 = 0x03;
    type Request = ();
    type Response = Infallible;
}

#[derive(Debug)]
pub enum AppletTunnel {}
#[sealed]
impl<'a> crate::Service<'a> for AppletTunnel {
    const IDENTIFIER: u8 = 0x04;
    type Request = applet::Tunnel<'a>;
    type Response = ();
}
