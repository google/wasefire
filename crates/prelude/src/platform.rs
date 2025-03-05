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
pub use wasefire_common::platform::Side;
#[cfg(feature = "api-platform")]
use wasefire_error::Error;
#[cfg(feature = "api-platform")]
use wasefire_sync::Lazy;

#[cfg(feature = "api-platform")]
use crate::{convert, convert_bool, convert_never};

#[cfg(feature = "api-platform-protocol")]
pub mod protocol;
#[cfg(feature = "api-platform-update")]
pub mod update;

/// Returns the serial of the platform.
#[cfg(feature = "api-platform")]
pub fn serial() -> &'static [u8] {
    fn init() -> &'static [u8] {
        let mut ptr = core::ptr::null_mut();
        let params = api::serial::Params { ptr: &mut ptr };
        let len = convert(unsafe { api::serial(params) }).unwrap();
        // SAFETY: If `len` is non-zero then `ptr` is the non-null result of `alloc(len, 1)`. The
        // scheduler traps the applet if `alloc()` returns null. The slice was fully initialized by
        // the scheduler.
        if len == 0 { &[] } else { unsafe { core::slice::from_raw_parts(ptr, len) } }
    }
    static SERIAL: Lazy<&'static [u8]> = Lazy::new(init);
    &SERIAL
}

/// Returns the running side of the platform.
#[cfg(feature = "api-platform")]
pub fn running_side() -> Side {
    let is_b = convert_bool(unsafe { api::running_side() }).unwrap();
    if is_b { Side::B } else { Side::A }
}

/// Returns the running version of the platform.
#[cfg(feature = "api-platform")]
pub fn running_version() -> &'static [u8] {
    static VERSION: Lazy<&'static [u8]> = Lazy::new(|| Box::leak(version(true).unwrap()));
    &VERSION
}

/// Returns the opposite version of the platform.
#[cfg(feature = "api-platform")]
pub fn opposite_version() -> Result<Box<[u8]>, Error> {
    version(false)
}

/// Reboots the device (thus platform and applets).
#[cfg(feature = "api-platform")]
pub fn reboot() -> ! {
    convert_never(unsafe { api::reboot() }).unwrap();
}

#[cfg(feature = "api-platform")]
fn version(running: bool) -> Result<Box<[u8]>, Error> {
    let mut ptr = core::ptr::null_mut();
    let params = api::version::Params { running: running as u32, ptr: &mut ptr };
    let len = convert(unsafe { api::version(params) })?;
    let ptr = core::ptr::slice_from_raw_parts_mut(ptr, len);
    // SAFETY: Similar as in `serial()` above.
    Ok(if len == 0 { Box::new([]) } else { unsafe { Box::from_raw(ptr) } })
}
