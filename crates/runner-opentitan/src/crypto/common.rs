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

// sw/device/lib/crypto/include/datatypes.h

use alloc::alloc::{alloc, dealloc};
use core::alloc::Layout;

use wasefire_error::{Code, Error};

use crate::error::unwrap_status;
use crate::hardened_bool;

// otcrypto_hash_mode_t
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum HashMode {
    Sha256 = 0x69b,
}

impl HashMode {
    pub fn to_c(self) -> i32 {
        self as i32
    }
}

// otcrypto_const_byte_buf_t
#[derive(Clone, Copy)]
#[repr(C)]
pub struct ConstByteBuf {
    pub data: *const u8,
    pub len: usize,
}

impl From<&[u8]> for ConstByteBuf {
    fn from(value: &[u8]) -> Self {
        ConstByteBuf { data: value.as_ptr(), len: value.len() }
    }
}

// otcrypto_byte_buf_t
#[derive(Clone, Copy)]
#[repr(C)]
pub struct ByteBuf {
    pub data: *mut u8,
    pub len: usize,
}

impl From<&mut [u8]> for ByteBuf {
    fn from(value: &mut [u8]) -> Self {
        ByteBuf { data: value.as_mut_ptr(), len: value.len() }
    }
}

// otcrypto_const_word32_buf_t
#[repr(C)]
pub struct ConstWord32Buf {
    pub data: *const u32,
    pub len: usize,
}

impl From<&[u32]> for ConstWord32Buf {
    fn from(value: &[u32]) -> Self {
        ConstWord32Buf { data: value.as_ptr(), len: value.len() }
    }
}

// otcrypto_word32_buf_t
#[repr(C)]
pub struct Word32Buf {
    pub data: *mut u32,
    pub len: usize,
}

impl From<&mut [u32]> for Word32Buf {
    fn from(value: &mut [u32]) -> Self {
        Word32Buf { data: value.as_mut_ptr(), len: value.len() }
    }
}

// otcrypto_hash_digest_t
#[repr(C)]
pub struct HashDigest {
    pub mode: i32,
    pub data: *mut u32,
    pub len: usize,
}

// otcrypto_key_mode_t
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum KeyMode {
    AesCbc = (KeyType::Aes.to_c() << 16 | AesKeyMode::Cbc.to_c()) as isize,
    HmacSha256 = (KeyType::Hmac.to_c() << 16 | HmacKeyMode::Sha256.to_c()) as isize,
}

impl KeyMode {
    const fn to_c(self) -> i32 {
        self as i32
    }

    fn key_length(self) -> usize {
        match self {
            KeyMode::AesCbc => 32,
            KeyMode::HmacSha256 => 64,
        }
    }
}

// otcrypto_key_type_t
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum KeyType {
    Aes = 0x8e9,
    Hmac = 0xe3f,
}

impl KeyType {
    const fn to_c(self) -> i32 {
        self as i32
    }
}

// otcrypto_aes_key_mode_t
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AesKeyMode {
    Cbc = 0xf3a,
}

impl AesKeyMode {
    const fn to_c(self) -> i32 {
        self as i32
    }
}

// otcrypto_hmac_key_mode_t
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum HmacKeyMode {
    Sha256 = 0x7fd,
}

impl HmacKeyMode {
    const fn to_c(self) -> i32 {
        self as i32
    }
}

// otcrypto_key_config_t
#[repr(C)]
pub struct KeyConfig {
    pub version: i32,
    pub key_mode: i32,
    pub key_length: usize,
    pub hw_backed: i32,
    pub exportable: i32,
    pub security_level: i32,
}

impl KeyConfig {
    pub fn new(key_mode: KeyMode) -> Self {
        KeyConfig {
            version: 0x7f4, // Version 1
            key_mode: key_mode.to_c(),
            key_length: key_mode.key_length(),
            hw_backed: hardened_bool::FALSE,
            exportable: hardened_bool::TRUE,
            security_level: SecurityLevel::Low.to_c(),
        }
    }
}

// otcrypto_key_security_level_t
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SecurityLevel {
    Low = 0x1e9,
    #[allow(dead_code)]
    Medium = 0xeab,
    #[allow(dead_code)]
    High = 0xa7e,
}

impl SecurityLevel {
    const fn to_c(self) -> i32 {
        self as i32
    }
}

// otcrypto_blinded_key_t
#[repr(C)]
pub struct BlindedKey {
    pub config: KeyConfig,
    pub keyblob_length: usize,
    pub keyblob: *mut u32, // owned
    pub checksum: u32,
}

unsafe impl Send for BlindedKey {}

impl Drop for BlindedKey {
    fn drop(&mut self) {
        unsafe { dealloc(self.keyblob as *mut u8, Self::layout(self.keyblob_length).unwrap()) };
    }
}

impl BlindedKey {
    pub fn import(
        config: KeyConfig, share0: ConstWord32Buf, share1: ConstWord32Buf,
    ) -> Result<Self, Error> {
        let keyblob_length = 4 * (share0.len + share1.len);
        let keyblob = unsafe { alloc(Self::layout(keyblob_length)?) as *mut u32 };
        let mut key = BlindedKey { config, keyblob_length, keyblob, checksum: 0 };
        let status = unsafe { otcrypto_import_blinded_key(share0, share1, &mut key) };
        unwrap_status(status)?;
        Ok(key)
    }

    fn layout(keyblob_length: usize) -> Result<Layout, Error> {
        Layout::from_size_align(keyblob_length, 4).map_err(|_| Error::user(Code::InvalidArgument))
    }
}

unsafe extern "C" {
    fn otcrypto_import_blinded_key(
        share0: ConstWord32Buf, share1: ConstWord32Buf, key: *mut BlindedKey,
    ) -> i32;
}
