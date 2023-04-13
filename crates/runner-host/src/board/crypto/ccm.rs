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

use aes::cipher::generic_array::GenericArray;
use aes::Aes128;
use ccm::aead::{consts, AeadInPlace};
use ccm::{Ccm, KeyInit};
use wasefire_board_api as board;

type NordicCcm = Ccm<Aes128, consts::U4, consts::U13>;

impl board::crypto::ccm::Api for &mut crate::board::Board {
    fn encrypt(
        &mut self, key: &[u8], iv: &[u8], clear: &[u8], cipher: &mut [u8],
    ) -> Result<(), board::Error> {
        let key = GenericArray::from_slice(key);
        cipher[.. clear.len()].copy_from_slice(clear);
        let mut nonce = [0; 13];
        nonce[5 ..].copy_from_slice(iv);
        let ccm = NordicCcm::new(key);
        match ccm.encrypt_in_place_detached(&nonce.into(), &[0], &mut cipher[.. clear.len()]) {
            Ok(tag) => {
                cipher[clear.len() ..].copy_from_slice(&tag);
                Ok(())
            }
            Err(_) => Err(board::Error::World),
        }
    }

    fn decrypt(
        &mut self, key: &[u8], iv: &[u8], cipher: &[u8], clear: &mut [u8],
    ) -> Result<(), board::Error> {
        let key = GenericArray::from_slice(key);
        clear.copy_from_slice(&cipher[.. clear.len()]);
        let mut nonce = [0; 13];
        nonce[5 ..].copy_from_slice(iv);
        let tag = &cipher[clear.len() ..];
        let ccm = NordicCcm::new(key);
        ccm.decrypt_in_place_detached(&nonce.into(), &[0], clear, tag.into())
            .map_err(|_| board::Error::World)
    }
}
