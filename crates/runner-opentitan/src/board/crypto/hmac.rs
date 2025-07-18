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

use crypto_common::{BlockSizeUser, KeySizeUser};
use digest::{FixedOutput, HashMarker, Key, KeyInit, MacMarker, Output, OutputSizeUser, Update};
use wasefire_board_api::Supported;
use wasefire_board_api::crypto::{GlobalError, WithError};
use wasefire_error::Error;

use crate::crypto::common::{HashMode, HmacKeyMode, KeyConfig, KeyMode, OwnedBlindedKey};
use crate::crypto::{hash, hmac};

pub struct Sha256(Option<hash::Context>);

impl Supported for Sha256 {}

impl Default for Sha256 {
    fn default() -> Self {
        Sha256(ERROR.record(hash::Context::init(HashMode::Sha256)))
    }
}

impl BlockSizeUser for Sha256 {
    type BlockSize = typenum::U64;
}

impl OutputSizeUser for Sha256 {
    type OutputSize = typenum::U32;
}

impl Update for Sha256 {
    fn update(&mut self, data: &[u8]) {
        if let Some(hash) = &mut self.0 {
            // TODO(https://github.com/rust-lang/rust-clippy/issues/13371): Remove when fixed.
            #[allow(clippy::single_match)]
            match ERROR.record(hash.update(data)) {
                None => self.0 = None,
                Some(()) => (),
            }
        }
    }
}

impl FixedOutput for Sha256 {
    fn finalize_into(self, out: &mut Output<Self>) {
        if let Some(hash) = self.0 {
            let mut digest = [0; 8];
            if ERROR.record(hash.finalize(&mut digest)) == Some(()) {
                out.copy_from_slice(bytemuck::bytes_of(&digest));
            }
        }
    }
}

impl HashMarker for Sha256 {}

impl WithError for Sha256 {
    fn with_error<T>(operation: impl FnOnce() -> T) -> Result<T, Error> {
        ERROR.with(operation)
    }
}

pub struct HmacSha256(Option<hmac::Context>);

impl Supported for HmacSha256 {}

impl KeySizeUser for HmacSha256 {
    type KeySize = typenum::U64;
}

impl OutputSizeUser for HmacSha256 {
    type OutputSize = typenum::U32;
}

impl KeyInit for HmacSha256 {
    fn new(key: &Key<Self>) -> Self {
        Self::new_from_slice(key).unwrap()
    }

    fn new_from_slice(key: &[u8]) -> Result<Self, digest::InvalidLength> {
        fn aux(key: &[u8]) -> Result<Option<hmac::Context>, Error> {
            let mut key_ = [0u32; 16];
            if key.len() <= 64 {
                bytemuck::bytes_of_mut(&mut key_)[.. key.len()].copy_from_slice(key);
            } else {
                let mut hash = hash::Context::init(HashMode::Sha256)?;
                hash.update(key)?;
                hash.finalize(&mut key_[.. 8])?;
            }
            let config = KeyConfig::new(KeyMode::Hmac(HmacKeyMode::Sha256));
            let key = OwnedBlindedKey::import(config, key_[..].into(), [0u32; 16][..].into())?;
            Ok(Some(hmac::Context::init(key)?))
        }
        match ERROR.record(aux(key)) {
            Some(Some(x)) => Ok(HmacSha256(Some(x))),
            Some(None) => Err(digest::InvalidLength),
            None => Ok(HmacSha256(None)),
        }
    }
}

impl Update for HmacSha256 {
    fn update(&mut self, data: &[u8]) {
        if let Some(hmac) = &mut self.0 {
            // TODO(https://github.com/rust-lang/rust-clippy/issues/13371): Remove when fixed.
            #[allow(clippy::single_match)]
            match ERROR.record(hmac.update(data)) {
                None => self.0 = None,
                Some(()) => (),
            }
        }
    }
}

impl FixedOutput for HmacSha256 {
    fn finalize_into(self, out: &mut Output<Self>) {
        if let Some(hmac) = self.0 {
            let mut tag = [0; 8];
            if ERROR.record(hmac.finalize(&mut tag)) == Some(()) {
                out.copy_from_slice(bytemuck::bytes_of(&tag));
            }
        }
    }
}

impl MacMarker for HmacSha256 {}

impl WithError for HmacSha256 {
    fn with_error<T>(operation: impl FnOnce() -> T) -> Result<T, Error> {
        ERROR.with(operation)
    }
}

static ERROR: GlobalError = GlobalError::new();
