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

use aes_gcm::{AeadInPlace, Aes256Gcm, KeyInit};
use wasefire_board_api as board;

impl board::crypto::gcm::Api for &mut crate::board::Board {
    fn encrypt(
        &mut self, key: &[u8; 32], iv: &[u8; 12], aad: &[u8], clear: &[u8], cipher: &mut [u8],
        tag: &mut [u8; 16],
    ) -> Result<(), board::Error> {
        let gcm = Aes256Gcm::new(key.into());
        cipher.copy_from_slice(clear);
        tag.copy_from_slice(
            &gcm.encrypt_in_place_detached(iv.into(), aad, cipher)
                .map_err(|_| board::Error::World)?,
        );
        Ok(())
    }

    fn decrypt(
        &mut self, key: &[u8; 32], iv: &[u8; 12], aad: &[u8], tag: &[u8; 16], cipher: &[u8],
        clear: &mut [u8],
    ) -> Result<(), board::Error> {
        let gcm = Aes256Gcm::new(key.into());
        clear.copy_from_slice(cipher);
        gcm.decrypt_in_place_detached(iv.into(), aad, clear, tag.into())
            .map_err(|_| board::Error::World)
    }
}
