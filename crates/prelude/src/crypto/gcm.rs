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

//! Provides AES-256-GCM.

use alloc::vec;
use alloc::vec::Vec;

use wasefire_applet_api::crypto::gcm as api;

use super::Error;

pub struct Cipher {
    pub text: Vec<u8>,
    pub tag: [u8; 16],
}

/// Encrypts and authenticates a cleartext.
pub fn encrypt(key: &[u8; 32], iv: &[u8; 12], aad: &[u8], clear: &[u8]) -> Result<Cipher, Error> {
    let mut cipher = Cipher { text: vec![0; clear.len()], tag: [0; 16] };
    let params = api::encrypt::Params {
        key: key.as_ptr(),
        iv: iv.as_ptr(),
        aad: aad.as_ptr(),
        aad_len: aad.len(),
        length: clear.len(),
        clear: clear.as_ptr(),
        cipher: cipher.text.as_mut_ptr(),
        tag: cipher.tag.as_mut_ptr(),
    };
    let api::encrypt::Results { res } = unsafe { api::encrypt(params) };
    Error::to_result(res)?;
    Ok(cipher)
}

/// Decrypts and authenticates a ciphertext.
pub fn decrypt(
    key: &[u8; 32], iv: &[u8; 12], aad: &[u8], cipher: &Cipher,
) -> Result<Vec<u8>, Error> {
    let mut clear = vec![0; cipher.text.len()];
    let params = api::decrypt::Params {
        key: key.as_ptr(),
        iv: iv.as_ptr(),
        aad: aad.as_ptr(),
        aad_len: aad.len(),
        tag: cipher.tag.as_ptr(),
        length: cipher.text.len(),
        cipher: cipher.text.as_ptr(),
        clear: clear.as_mut_ptr(),
    };
    let api::decrypt::Results { res } = unsafe { api::decrypt(params) };
    Error::to_result(res)?;
    Ok(clear)
}
