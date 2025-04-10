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
use wasefire_board_api::crypto::WithError;
use wasefire_error::Error;
use wasefire_sync::TakeCell;

use crate::hmac::Hmac;

pub struct Sha256(Option<Hmac>);

impl Supported for Sha256 {}

impl Default for Sha256 {
    fn default() -> Self {
        Sha256(ERROR.record(Hmac::start(None)))
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
            hash.update(data)
        }
    }
}

impl FixedOutput for Sha256 {
    fn finalize_into(self, out: &mut Output<Self>) {
        if let Some(hash) = self.0 {
            ERROR.record(hash.finalize(out.as_mut()));
        }
    }
}

impl HashMarker for Sha256 {}

impl WithError for Sha256 {
    fn with_error<T>(operation: impl FnOnce() -> T) -> Result<T, Error> {
        ERROR.with(operation)
    }
}

pub struct HmacSha256(Option<Hmac>);

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
        fn aux(key: &[u8]) -> Result<Option<Hmac>, Error> {
            let mut key_ = [0; 64];
            if key.len() <= 64 {
                key_[.. key.len()].copy_from_slice(key);
            } else {
                let mut hash = Hmac::start(None)?;
                hash.update(key);
                hash.finalize(key_.first_chunk_mut().unwrap())?;
            }
            Ok(Some(Hmac::start(Some(&key_))?))
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
            hmac.update(data);
        }
    }
}

impl FixedOutput for HmacSha256 {
    fn finalize_into(self, out: &mut Output<Self>) {
        if let Some(hmac) = self.0 {
            ERROR.record(hmac.finalize(out.as_mut()));
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

struct GlobalError(TakeCell<Result<(), Error>>);

impl GlobalError {
    const fn new() -> Self {
        GlobalError(TakeCell::new(None))
    }

    fn with<T>(&self, operation: impl FnOnce() -> T) -> Result<T, Error> {
        self.0.put(Ok(()));
        let result = operation();
        self.0.take().map(|()| result)
    }

    fn record<T>(&self, x: Result<T, Error>) -> Option<T> {
        x.inspect_err(|e| self.0.with(|x| *x = Err(*e))).ok()
    }
}
