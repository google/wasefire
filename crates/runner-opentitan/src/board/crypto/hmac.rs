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

use crypto_common::BlockSizeUser;
use digest::{FixedOutput, HashMarker, Output, OutputSizeUser, Update};
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
