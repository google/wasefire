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

//! Provides hash functions.

use wasefire_applet_api::crypto::hash as api;

pub use self::api::Algorithm;
use super::Error;

/// Hashing context.
pub struct Digest {
    /// The hashing context identifier.
    id: usize,

    /// The digest length in bytes.
    len: usize,
}

impl Digest {
    /// Creates a new hashing context for the specified algorithm.
    pub fn new(algorithm: Algorithm) -> Result<Self, Error> {
        if !is_supported(algorithm) {
            return Err(Error::Unsupported);
        }
        let params = api::initialize::Params { algorithm: algorithm as usize };
        let api::initialize::Results { id } = unsafe { api::initialize(params) };
        let id = Error::to_result(id)?;
        let len = algorithm.digest_len();
        Ok(Self { id, len })
    }

    /// Updates the hashing context with the provided data.
    pub fn update(&mut self, data: &[u8]) {
        let params = api::update::Params { id: self.id, data: data.as_ptr(), length: data.len() };
        unsafe { api::update(params) };
    }

    /// Finalizes the hashing context and writes the associated digest.
    pub fn finalize(self, digest: &mut [u8]) -> Result<(), Error> {
        if digest.len() != self.len {
            return Err(Error::InvalidArgument);
        }
        let params = api::finalize::Params { id: self.id, digest: digest.as_mut_ptr() };
        let api::finalize::Results { res } = unsafe { api::finalize(params) };
        Error::to_result(res).map(|_| ())
    }

    /// Writes the hash of the data for the given algorithm in the digest.
    pub fn digest(algorithm: Algorithm, data: &[u8], digest: &mut [u8]) -> Result<(), Error> {
        let mut context = Self::new(algorithm)?;
        context.update(data);
        context.finalize(digest)
    }
}

/// Returns the SHA-256 of the provided data.
pub fn sha256(data: &[u8]) -> Result<[u8; 32], Error> {
    let mut digest = [0; 32];
    Digest::digest(Algorithm::Sha256, data, &mut digest)?;
    Ok(digest)
}

/// Whether a hash algorithm is supported.
pub fn is_supported(algorithm: Algorithm) -> bool {
    let params = api::is_supported::Params { algorithm: algorithm as usize };
    let api::is_supported::Results { supported } = unsafe { api::is_supported(params) };
    supported != 0
}
