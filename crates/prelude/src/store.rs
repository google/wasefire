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

use alloc::boxed::Box;

use api::store as api;

/// Errors returned by storage operations.
pub use self::api::Error;

/// Inserts an entry in the store.
///
/// The `key` argument must be a small integer (currently less than 4096). The `value` argument is
/// the slice to associate with this key. If there was already a value, it is overwritten.
/// Overwritten values are zeroized from flash.
pub fn insert(key: usize, value: &[u8]) -> Result<(), Error> {
    let params = api::insert::Params { key, ptr: value.as_ptr(), len: value.len() };
    let api::insert::Results { res } = unsafe { api::insert(params) };
    Error::to_result(res)?;
    Ok(())
}

/// Removes an entry from the store.
///
/// If there was not value associated with the `key` argument, this is a no-op. Otherwise the value
/// is zeroized from flash and the key is not associated.
pub fn remove(key: usize) -> Result<(), Error> {
    let params = api::remove::Params { key };
    let api::remove::Results { res } = unsafe { api::remove(params) };
    Error::to_result(res)?;
    Ok(())
}

/// Returns the value associated to a key, if any.
pub fn find(key: usize) -> Result<Option<Box<[u8]>>, Error> {
    find_impl(key)
}

#[cfg(feature = "multivalue")]
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
fn find_impl(key: usize) -> Result<Option<Box<[u8]>>, Error> {
    let mut ptr = core::ptr::null_mut();
    let mut len = 0;
    let params = api::find::Params { key, ptr: &mut ptr, len: &mut len };
    let api::find::Results { res } = unsafe { api::find(params) };
    match Error::to_result(res)? {
        0 => Ok(None),
        1 => {
            let ptr = unsafe { core::slice::from_raw_parts_mut(ptr, len) };
            Ok(Some(unsafe { Box::from_raw(ptr) }))
        }
        _ => unreachable!(),
    }
}
