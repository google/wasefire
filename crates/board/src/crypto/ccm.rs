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

//! AES-CCM according to Bluetooth.

use crate::Error;

/// AES-CCM interface (according to Bluetooth).
pub trait Api {
    /// Encrypts a clear-text to a cipher-text given a key and IV.
    ///
    /// The key must be 16 bytes. The IV must be 8 bytes. The cipher-text must be 4 bytes longer
    /// than the clear-text. The clear-text must not be longer than 251 bytes.
    fn encrypt(
        &mut self, key: &[u8], iv: &[u8], clear: &[u8], cipher: &mut [u8],
    ) -> Result<(), Error>;

    /// Decrypts a cipher-text to a clear-text given the key and IV.
    ///
    /// The key must be 16 bytes. The IV must be 8 bytes. The cipher-text must be 4 bytes longer
    /// than the clear-text. The clear-text must not be longer than 251 bytes.
    fn decrypt(
        &mut self, key: &[u8], iv: &[u8], cipher: &[u8], clear: &mut [u8],
    ) -> Result<(), Error>;
}

impl Api for ! {
    fn encrypt(&mut self, _: &[u8], _: &[u8], _: &[u8], _: &mut [u8]) -> Result<(), Error> {
        unreachable!()
    }

    fn decrypt(&mut self, _: &[u8], _: &[u8], _: &[u8], _: &mut [u8]) -> Result<(), Error> {
        unreachable!()
    }
}
