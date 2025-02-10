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

//! Provides API for random number generation.

use alloc::boxed::Box;
use core::mem::MaybeUninit;

use wasefire_applet_api::rng as api;

use crate::{Error, convert_unit};

/// Returns a slice of random bytes.
pub fn bytes(len: usize) -> Result<Box<[u8]>, Error> {
    let mut buf = Box::new_uninit_slice(len);
    fill_uninit_bytes(&mut buf)?;
    // SAFETY: `fill_uninit_bytes()` only succeeds if all bytes are initialized.
    Ok(unsafe { buf.assume_init() })
}

/// Returns an array of random bytes.
pub fn bytes_array<const N: usize>() -> Result<[u8; N], Error> {
    let mut buf = MaybeUninit::uninit_array();
    fill_uninit_bytes(&mut buf)?;
    // SAFETY: `fill_uninit_bytes()` only succeeds if all bytes are initialized.
    Ok(unsafe { MaybeUninit::array_assume_init(buf) })
}

/// Fills a slice with random bytes.
///
/// Prefer [`bytes()`] if you don't already have an allocation.
pub fn fill_bytes(buf: &mut [u8]) -> Result<(), Error> {
    // SAFETY: `fill_uninit_bytes()` only writes initialized bytes.
    let buf = unsafe {
        core::slice::from_raw_parts_mut(buf.as_mut_ptr() as *mut MaybeUninit<u8>, buf.len())
    };
    fill_uninit_bytes(buf)
}

fn fill_uninit_bytes(buf: &mut [MaybeUninit<u8>]) -> Result<(), Error> {
    let params = api::fill_bytes::Params { ptr: buf.as_mut_ptr() as *mut u8, len: buf.len() };
    convert_unit(unsafe { api::fill_bytes(params) })
}
