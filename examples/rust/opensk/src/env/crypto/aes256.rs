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
// limitations under the License.use opensk_lib::api::clock::Clock;

use opensk_lib::api::crypto::aes256::Aes256;
use opensk_lib::api::crypto::{AES_BLOCK_SIZE, AES_KEY_SIZE};

use crate::env::WasefireEnv;

impl Aes256 for WasefireEnv {
    fn new(_key: &[u8; AES_KEY_SIZE]) -> Self {
        todo!()
    }

    fn encrypt_block(&self, _block: &mut [u8; AES_BLOCK_SIZE]) {
        todo!()
    }

    fn decrypt_block(&self, _block: &mut [u8; AES_BLOCK_SIZE]) {
        todo!()
    }

    fn encrypt_cbc(&self, _iv: &[u8; AES_BLOCK_SIZE], _plaintext: &mut [u8]) {
        todo!()
    }

    fn decrypt_cbc(&self, _iv: &[u8; AES_BLOCK_SIZE], _ciphertext: &mut [u8]) {
        todo!()
    }
}
