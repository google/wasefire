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

//! AES-256-GCM.

use crate::Error;

/// AES-256-GCM interface.
pub trait Api {
    /// Encrypts and authenticates a clear text with associated data given a key and IV.
    ///
    /// The clear- and cipher-texts must have the same length.
    fn encrypt(
        &mut self, key: &[u8; 32], iv: &[u8; 12], aad: &[u8], clear: &[u8], cipher: &mut [u8],
        tag: &mut [u8; 16],
    ) -> Result<(), Error>;

    /// Decrypts and authenticates a cipher text with associated data given a key and IV.
    ///
    /// The cipher- and clear-texts must have the same length.
    fn decrypt(
        &mut self, key: &[u8; 32], iv: &[u8; 12], aad: &[u8], tag: &[u8; 16], cipher: &[u8],
        clear: &mut [u8],
    ) -> Result<(), Error>;
}

impl Api for ! {
    fn encrypt(
        &mut self, _: &[u8; 32], _: &[u8; 12], _: &[u8], _: &[u8], _: &mut [u8], _: &mut [u8; 16],
    ) -> Result<(), Error> {
        unreachable!()
    }

    fn decrypt(
        &mut self, _: &[u8; 32], _: &[u8; 12], _: &[u8], _: &[u8; 16], _: &[u8], _: &mut [u8],
    ) -> Result<(), Error> {
        unreachable!()
    }
}
