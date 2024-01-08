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
use wasefire_applet_api::platform as api;

#[cfg(feature = "api-platform")]
use crate::convert;

#[cfg(feature = "api-platform-update")]
pub mod update;

/// Returns the version of the platform.
#[cfg(feature = "api-platform")]
pub fn version() -> &'static [u8] {
    use wasefire_sync::Mutex;
    static VERSION: Mutex<Option<&'static [u8]>> = Mutex::new(None);
    let mut guard = VERSION.lock();
    if let Some(version) = *guard {
        return version;
    }
    let params = api::version::Params { ptr: core::ptr::null_mut(), len: 0 };
    let len = convert(unsafe { api::version(params) }).unwrap();
    let mut version = alloc::vec![0; len];
    let params = api::version::Params { ptr: version[..].as_mut_ptr(), len };
    convert(unsafe { api::version(params) }).unwrap();
    let version = alloc::boxed::Box::leak(version.into_boxed_slice());
    *guard = Some(version);
    version
}

/// Reboots the device (thus platform and applets).
#[cfg(feature = "api-platform")]
pub fn reboot() -> ! {
    let res = convert(unsafe { api::reboot() }).unwrap();
    unreachable!("reboot() returned {res}");
}
