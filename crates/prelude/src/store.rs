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
use alloc::vec::Vec;

#[cfg(feature = "api-store")]
use wasefire_applet_api::store as api;

#[cfg(feature = "api-store")]
use crate::{Error, convert, convert_bool, convert_unit};

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
    convert_unit(unsafe { api::insert(params) })
}

/// Removes an entry from the store.
///
/// If there was not value associated with the `key` argument, this is a no-op. Otherwise the value
/// is zeroized from flash and the key is not associated.
#[cfg(feature = "api-store")]
pub fn remove(key: usize) -> Result<(), Error> {
    let params = api::remove::Params { key };
    convert_unit(unsafe { api::remove(params) })
}

/// Returns the value associated to a key, if any.
#[cfg(feature = "api-store")]
pub fn find(key: usize) -> Result<Option<Box<[u8]>>, Error> {
    let mut ptr = core::ptr::null_mut();
    let mut len = 0;
    let params = api::find::Params { key, ptr: &mut ptr, len: &mut len };
    if convert_bool(unsafe { api::find(params) })? {
        let ptr = core::ptr::slice_from_raw_parts_mut(ptr, len);
        Ok(Some(unsafe { Box::from_raw(ptr) }))
    } else {
        Ok(None)
    }
}

/// Returns the keys of the entries in the store.
#[cfg(feature = "api-store")]
pub fn keys() -> Result<Vec<u16>, Error> {
    let mut ptr = core::ptr::null_mut();
    let params = api::keys::Params { ptr: &mut ptr };
    let len = convert(unsafe { api::keys(params) })?;
    if len == 0 {
        Ok(Vec::new())
    } else {
        Ok(unsafe { Vec::from_raw_parts(ptr as *mut u16, len, len) })
    }
}

/// Clears the store, removing all entries.
#[cfg(feature = "api-store")]
pub fn clear() -> Result<(), Error> {
    convert_unit(unsafe { api::clear() })
}
