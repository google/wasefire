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

//! Provides API for to update the platform.

use alloc::boxed::Box;

use wasefire_applet_api::platform::update as api;

use crate::{convert_unit, Error};

/// Returns whether platform update is supported.
pub fn is_supported() -> bool {
    let api::is_supported::Results { supported } = unsafe { api::is_supported() };
    supported != 0
}

/// Returns the metadata of the platform.
///
/// This typically contains the version and side (A or B) of the running platform.
pub fn metadata() -> Result<Box<[u8]>, Error> {
    let mut ptr = core::ptr::null_mut();
    let mut len = 0;
    let params = api::metadata::Params { ptr: &mut ptr, len: &mut len };
    let api::metadata::Results { res } = unsafe { api::metadata(params) };
    convert_unit(res)?;
    let ptr = unsafe { core::slice::from_raw_parts_mut(ptr, len) };
    Ok(unsafe { Box::from_raw(ptr) })
}

/// Starts a platform update process.
///
/// During a dry-run, any mutable operation is skipped and only checks are performed.
pub fn initialize(dry_run: bool) -> Result<(), Error> {
    let params = api::initialize::Params { dry_run: dry_run as usize };
    let api::initialize::Results { res } = unsafe { api::initialize(params) };
    convert_unit(res)
}

/// Processes the next chunk of a platform update.
pub fn process(chunk: &[u8]) -> Result<(), Error> {
    let params = api::process::Params { ptr: chunk.as_ptr(), len: chunk.len() };
    let api::process::Results { res } = unsafe { api::process(params) };
    convert_unit(res)
}

/// Finalizes a platform update process.
///
/// This function will reboot when the update is successful and thus only returns in case of errors
/// or in dry-run mode.
pub fn finalize() -> Result<(), Error> {
    let api::finalize::Results { res } = unsafe { api::finalize() };
    convert_unit(res)
}
