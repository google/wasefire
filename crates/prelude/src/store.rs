// Copyright 2022 Google LLC
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

//! Provides API for persistent storage.

#[cfg(feature = "api-store")]
use alloc::boxed::Box;

#[cfg(feature = "api-store")]
use wasefire_applet_api::store as api;

#[cfg(feature = "api-store")]
use crate::{convert_unit, Error};

#[cfg(feature = "api-store-fragment")]
pub mod fragment;

/// Inserts an entry in the store.
///
/// The `key` argument must be a small integer (currently less than 4096). The `value` argument is
/// the slice to associate with this key. If there was already a value, it is overwritten.
/// Overwritten values are zeroized from flash.
#[cfg(feature = "api-store")]
pub fn insert(key: usize, value: &[u8]) -> Result<(), Error> {
    let params = api::insert::Params { key, ptr: value.as_ptr(), len: value.len() };
    let api::insert::Results { res } = unsafe { api::insert(params) };
    convert_unit(res)
}

/// Removes an entry from the store.
///
/// If there was not value associated with the `key` argument, this is a no-op. Otherwise the value
/// is zeroized from flash and the key is not associated.
#[cfg(feature = "api-store")]
pub fn remove(key: usize) -> Result<(), Error> {
    let params = api::remove::Params { key };
    let api::remove::Results { res } = unsafe { api::remove(params) };
    convert_unit(res)
}

/// Returns the value associated to a key, if any.
#[cfg(feature = "api-store")]
pub fn find(key: usize) -> Result<Option<Box<[u8]>>, Error> {
    find_impl(key)
}

#[cfg(feature = "multivalue")]
#[cfg(feature = "api-store")]
fn find_impl(key: usize) -> Result<Option<Box<[u8]>>, Error> {
    let params = api::find::Params { key };
    let api::find::Results { ptr, len } = unsafe { api::find(params) };
    let len = Error::to_result(len)?;
    if ptr.is_null() {
        return Ok(None);
    }
    let ptr = unsafe { core::slice::from_raw_parts_mut(ptr, len) };
    Ok(Some(unsafe { Box::from_raw(ptr) }))
}

#[cfg(not(feature = "multivalue"))]
#[cfg(feature = "api-store")]
fn find_impl(key: usize) -> Result<Option<Box<[u8]>>, Error> {
    use crate::convert_bool;

    let mut ptr = core::ptr::null_mut();
    let mut len = 0;
    let params = api::find::Params { key, ptr: &mut ptr, len: &mut len };
    let api::find::Results { res } = unsafe { api::find(params) };
    if convert_bool(res)? {
        let ptr = unsafe { core::slice::from_raw_parts_mut(ptr, len) };
        Ok(Some(unsafe { Box::from_raw(ptr) }))
    } else {
        Ok(None)
    }
}
