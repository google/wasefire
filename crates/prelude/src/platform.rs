// Copyright 2023 Google LLC
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

//! Provides API to interact with the platform.

#[cfg(feature = "api-platform")]
use alloc::boxed::Box;

#[cfg(feature = "api-platform")]
use wasefire_applet_api::platform as api;
#[cfg(feature = "api-platform")]
use wasefire_sync::Lazy;

#[cfg(feature = "api-platform")]
use crate::{convert, convert_never};

#[cfg(feature = "api-platform-protocol")]
pub mod protocol;
#[cfg(feature = "api-platform-update")]
pub mod update;

/// Returns the serial of the platform.
#[cfg(feature = "api-platform")]
pub fn serial() -> &'static [u8] {
    fn init() -> Box<[u8]> {
        let mut ptr = core::ptr::null_mut();
        let params = api::serial::Params { ptr: &mut ptr };
        let len = convert(unsafe { api::serial(params) }).unwrap();
        unsafe { Box::from_raw(core::slice::from_raw_parts_mut(ptr, len)) }
    }
    static SERIAL: Lazy<Box<[u8]>> = Lazy::new(init);
    &SERIAL
}

/// Returns the version of the platform.
#[cfg(feature = "api-platform")]
pub fn version() -> &'static [u8] {
    fn init() -> Box<[u8]> {
        let mut ptr = core::ptr::null_mut();
        let params = api::version::Params { ptr: &mut ptr };
        let len = convert(unsafe { api::version(params) }).unwrap();
        unsafe { Box::from_raw(core::slice::from_raw_parts_mut(ptr, len)) }
    }
    static VERSION: Lazy<Box<[u8]>> = Lazy::new(init);
    &VERSION
}

/// Reboots the device (thus platform and applets).
#[cfg(feature = "api-platform")]
pub fn reboot() -> ! {
    convert_never(unsafe { api::reboot() }).unwrap();
}
