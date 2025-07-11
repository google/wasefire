// Copyright 2025 Google LLC
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

// sw/device/lib/crypto/include/aes.h

use wasefire_error::{Code, Error};

use crate::crypto::common::{BlindedKey, ByteBuf, ConstByteBuf, KeyConfig, KeyMode, Word32Buf};
use crate::error::unwrap_status;

pub fn encrypt_cbc(key: &[u8; 32], iv: &[u8; 16], blocks: &mut [u8]) -> Result<(), Error> {
    if !blocks.len().is_multiple_of(16) {
        return Err(Error::user(Code::InvalidLength));
    }
    let key = convert_key(key)?;
    let mut iv: [u32; 4] = bytemuck::cast(*iv);
    unwrap_status(unsafe {
        otcrypto_aes(
            key,
            (&mut iv[..]).into(),
            Mode::Cbc.to_c(),
            Operation::Encrypt.to_c(),
            blocks[..].into(),
            Padding::Null.to_c(),
            blocks.into(),
        )
    })?;
    Ok(())
}

pub fn decrypt_cbc(key: &[u8; 32], iv: &[u8; 16], blocks: &mut [u8]) -> Result<(), Error> {
    if !blocks.len().is_multiple_of(16) {
        return Err(Error::user(Code::InvalidLength));
    }
    let key = convert_key(key)?;
    let mut iv: [u32; 4] = bytemuck::cast(*iv);
    unwrap_status(unsafe {
        otcrypto_aes(
            key,
            (&mut iv[..]).into(),
            Mode::Cbc.to_c(),
            Operation::Decrypt.to_c(),
            blocks[..].into(),
            Padding::Null.to_c(),
            blocks.into(),
        )
    })?;
    Ok(())
}

fn convert_key(key: &[u8; 32]) -> Result<BlindedKey, Error> {
    let config = KeyConfig::new(KeyMode::AesCbc);
    let key: [u32; 8] = bytemuck::cast(*key);
    BlindedKey::import(config, key[..].into(), [0u32; 8][..].into())
}

// otcrypto_aes_mode_t
#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    Cbc = 0x45d,
}

impl Mode {
    pub fn to_c(self) -> i32 {
        self as i32
    }
}

// otcrypto_aes_operation_t
#[derive(Clone, Copy, PartialEq, Eq)]
enum Operation {
    Encrypt = 0x2b6,
    Decrypt = 0x5f0,
}

impl Operation {
    pub fn to_c(self) -> i32 {
        self as i32
    }
}

// otcrypto_aes_padding_t
#[derive(Clone, Copy, PartialEq, Eq)]
enum Padding {
    Null = 0x8ce,
}

impl Padding {
    pub fn to_c(self) -> i32 {
        self as i32
    }
}

unsafe extern "C" {
    fn otcrypto_aes(
        key: BlindedKey, iv: Word32Buf, mode: i32, operation: i32, input: ConstByteBuf,
        padding: i32, output: ByteBuf,
    ) -> i32;
}
