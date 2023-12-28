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

//! Support for fragmented entries.

use alloc::boxed::Box;
use core::ops::Range;

use wasefire_applet_api::store::fragment as api;

use crate::{convert_bool, convert_unit, Error};

/// Inserts an entry in the store.
///
/// The entry will be fragmented over multiple keys within the provided range as needed.
///
/// The range must be non-empty and end before 4096. The `value` argument is the slice to associate
/// with this key. If there was already a value, it is overwritten. Overwritten values are zeroized
/// from flash.
pub fn insert(keys: Range<usize>, value: &[u8]) -> Result<(), Error> {
    let params =
        api::insert::Params { keys: encode_keys(keys)?, ptr: value.as_ptr(), len: value.len() };
    let api::insert::Results { res } = unsafe { api::insert(params) };
    convert_unit(res)
}

/// Removes an entry from the store.
///
/// All fragments from the range of keys will be deleted.
///
/// If there was not value associated with the `key` argument, this is a no-op. Otherwise the value
/// is zeroized from flash and the key is not associated.
pub fn remove(keys: Range<usize>) -> Result<(), Error> {
    let params = api::remove::Params { keys: encode_keys(keys)? };
    let api::remove::Results { res } = unsafe { api::remove(params) };
    convert_unit(res)
}

/// Returns the value associated to a key, if any.
///
/// The entry may be fragmented withen the provided range.
pub fn find(keys: Range<usize>) -> Result<Option<Box<[u8]>>, Error> {
    find_impl(keys)
}

fn find_impl(keys: Range<usize>) -> Result<Option<Box<[u8]>>, Error> {
    let mut ptr = core::ptr::null_mut();
    let mut len = 0;
    let params = api::find::Params { keys: encode_keys(keys)?, ptr: &mut ptr, len: &mut len };
    let api::find::Results { res } = unsafe { api::find(params) };
    if convert_bool(res)? {
        let ptr = unsafe { core::slice::from_raw_parts_mut(ptr, len) };
        Ok(Some(unsafe { Box::from_raw(ptr) }))
    } else {
        Ok(None)
    }
}

fn encode_keys(keys: Range<usize>) -> Result<u32, Error> {
    let start = u16::try_from(keys.start).map_err(|_| Error::user(0))? as u32;
    let end = u16::try_from(keys.end).map_err(|_| Error::user(0))? as u32;
    Ok(end << 16 | start)
}
