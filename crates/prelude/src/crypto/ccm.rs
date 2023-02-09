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

//! Provides AES-CCM as defined by Bluetooth.

use alloc::vec;
use alloc::vec::Vec;

use api::crypto::ccm as api;

use super::Error;

/// Returns the ciphertext given a cleartext, a key, and a nonce.
pub fn encrypt(key: &[u8; 16], nonce: &[u8; 8], clear: &[u8]) -> Result<Vec<u8>, Error> {
    let len = clear.len();
    let mut cipher = vec![0; len as usize + 4];
    let params = api::encrypt::Params {
        key: key.as_ptr(),
        iv: nonce.as_ptr(),
        len,
        clear: clear.as_ptr(),
        cipher: cipher.as_mut_ptr(),
    };
    let api::encrypt::Results { res } = unsafe { api::encrypt(params) };
    Error::to_result(res)?;
    Ok(cipher)
}

/// Returns the cleartext given a ciphertext, a key, and a nonce.
pub fn decrypt(key: &[u8; 16], nonce: &[u8; 8], cipher: &[u8]) -> Result<Vec<u8>, Error> {
    let len = cipher.len() - 4;
    let mut clear = vec![0; len as usize];
    let params = api::decrypt::Params {
        key: key.as_ptr(),
        iv: nonce.as_ptr(),
        len,
        cipher: cipher.as_ptr(),
        clear: clear.as_mut_ptr(),
    };
    let api::decrypt::Results { res } = unsafe { api::decrypt(params) };
    Error::to_result(res)?;
    Ok(clear)
}
