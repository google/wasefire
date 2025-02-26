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

//! Provides AES-256-CBC.

use wasefire_applet_api::crypto::cbc as api;

use crate::{Error, convert_bool, convert_unit};

/// Whether AES-256-CBC is supported.
pub fn is_supported() -> bool {
    convert_bool(unsafe { api::is_supported() }).unwrap_or(false)
}

/// Encrypts a sequence of blocks given a key and an IV.
pub fn encrypt(key: &[u8; 32], iv: &[u8; 16], blocks: &mut [u8]) -> Result<(), Error> {
    let params = api::encrypt::Params {
        key: key.as_ptr(),
        iv: iv.as_ptr(),
        ptr: blocks.as_mut_ptr(),
        len: blocks.len(),
    };
    convert_unit(unsafe { api::encrypt(params) })
}

/// Decrypts a sequence of blocks given a key and an IV.
pub fn decrypt(key: &[u8; 32], iv: &[u8; 16], blocks: &mut [u8]) -> Result<(), Error> {
    let params = api::decrypt::Params {
        key: key.as_ptr(),
        iv: iv.as_ptr(),
        ptr: blocks.as_mut_ptr(),
        len: blocks.len(),
    };
    convert_unit(unsafe { api::decrypt(params) })
}
