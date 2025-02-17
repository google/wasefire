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

pub struct Impl {
    // TODO: Zeroize.
    key: [u8; 32],
}

impl Aes256 for Impl {
    fn new(key: &[u8; AES_KEY_SIZE]) -> Self {
        Impl { key: *key }
    }

    fn encrypt_block(&self, block: &mut [u8; AES_BLOCK_SIZE]) {
        wasefire::crypto::cbc::encrypt(&self.key, &ZERO_IV, block).unwrap()
    }

    fn decrypt_block(&self, block: &mut [u8; AES_BLOCK_SIZE]) {
        wasefire::crypto::cbc::decrypt(&self.key, &ZERO_IV, block).unwrap()
    }

    fn encrypt_cbc(&self, iv: &[u8; AES_BLOCK_SIZE], blocks: &mut [u8]) {
        wasefire::crypto::cbc::encrypt(&self.key, iv, blocks).unwrap()
    }

    fn decrypt_cbc(&self, iv: &[u8; AES_BLOCK_SIZE], blocks: &mut [u8]) {
        wasefire::crypto::cbc::decrypt(&self.key, iv, blocks).unwrap()
    }
}

const ZERO_IV: [u8; 16] = [0; 16];
