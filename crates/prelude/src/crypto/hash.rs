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

#[cfg(feature = "rust-crypto")]
pub use rust_crypto::*;
use wasefire_applet_api::crypto::hash as api;

pub use self::api::Algorithm;
use super::Error;

/// Hashing context.
pub struct Digest {
    /// The hashing context identifier.
    ///
    /// Finalized when -1 (as usize). This is used to know whether Drop should finalize.
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
    pub fn finalize(mut self, digest: &mut [u8]) -> Result<(), Error> {
        if digest.len() != self.len {
            return Err(Error::InvalidArgument);
        }
        let params = api::finalize::Params { id: self.id, digest: digest.as_mut_ptr() };
        let api::finalize::Results { res } = unsafe { api::finalize(params) };
        self.id = usize::MAX;
        Error::to_result(res).map(|_| ())
    }

    /// Writes the hash of the data for the given algorithm in the digest.
    pub fn digest(algorithm: Algorithm, data: &[u8], digest: &mut [u8]) -> Result<(), Error> {
        let mut context = Self::new(algorithm)?;
        context.update(data);
        context.finalize(digest)
    }
}

impl Drop for Digest {
    fn drop(&mut self) {
        if self.id == usize::MAX {
            // Already finalized.
            return;
        }
        let params = api::finalize::Params { id: self.id, digest: core::ptr::null_mut() };
        unsafe { api::finalize(params) };
    }
}

/// Hmac context.
pub struct Hmac {
    /// The hmac context identifier.
    ///
    /// Finalized when -1 (as usize). This is used to know whether Drop should finalize.
    id: usize,

    /// The hmac length in bytes.
    len: usize,
}

impl Hmac {
    /// Creates a new hmac context for the specified algorithm.
    pub fn new(algorithm: Algorithm, key: &[u8]) -> Result<Self, Error> {
        if !is_hmac_supported(algorithm) {
            return Err(Error::Unsupported);
        }
        let params = api::hmac_initialize::Params {
            algorithm: algorithm as usize,
            key: key.as_ptr(),
            key_len: key.len(),
        };
        let api::hmac_initialize::Results { id } = unsafe { api::hmac_initialize(params) };
        let id = Error::to_result(id)?;
        let len = algorithm.digest_len();
        Ok(Self { id, len })
    }

    /// Updates the hmac context with the provided data.
    pub fn update(&mut self, data: &[u8]) {
        let params =
            api::hmac_update::Params { id: self.id, data: data.as_ptr(), length: data.len() };
        unsafe { api::hmac_update(params) };
    }

    /// Finalizes the hmac context and writes the associated hmac.
    pub fn finalize(mut self, hmac: &mut [u8]) -> Result<(), Error> {
        if hmac.len() != self.len {
            return Err(Error::InvalidArgument);
        }
        let params = api::hmac_finalize::Params { id: self.id, hmac: hmac.as_mut_ptr() };
        let api::hmac_finalize::Results { res } = unsafe { api::hmac_finalize(params) };
        self.id = usize::MAX;
        Error::to_result(res).map(|_| ())
    }

    /// Writes the hmac of the data for the given algorithm.
    pub fn hmac(
        algorithm: Algorithm, key: &[u8], data: &[u8], hmac: &mut [u8],
    ) -> Result<(), Error> {
        let mut context = Self::new(algorithm, key)?;
        context.update(data);
        context.finalize(hmac)
    }
}

impl Drop for Hmac {
    fn drop(&mut self) {
        if self.id == usize::MAX {
            // Already finalized.
            return;
        }
        let params = api::hmac_finalize::Params { id: self.id, hmac: core::ptr::null_mut() };
        unsafe { api::hmac_finalize(params) };
    }
}

/// Returns the SHA-256 of the provided data.
pub fn sha256(data: &[u8]) -> Result<[u8; 32], Error> {
    let mut digest = [0; 32];
    Digest::digest(Algorithm::Sha256, data, &mut digest)?;
    Ok(digest)
}

/// Returns the HMAC-SHA-256 of the provided data.
pub fn hmac_sha256(key: &[u8], data: &[u8]) -> Result<[u8; 32], Error> {
    let mut hmac = [0; 32];
    Hmac::hmac(Algorithm::Sha256, key, data, &mut hmac)?;
    Ok(hmac)
}

/// Whether a hash algorithm is supported.
pub fn is_supported(algorithm: Algorithm) -> bool {
    let params = api::is_supported::Params { algorithm: algorithm as usize };
    let api::is_supported::Results { supported } = unsafe { api::is_supported(params) };
    supported != 0
}

/// Whether a hash algorithm is supported for HMAC.
pub fn is_hmac_supported(algorithm: Algorithm) -> bool {
    let params = api::is_hmac_supported::Params { algorithm: algorithm as usize };
    let api::is_hmac_supported::Results { supported } = unsafe { api::is_hmac_supported(params) };
    supported != 0
}

#[cfg(feature = "rust-crypto")]
mod rust_crypto {
    use crypto_common::{KeyInit, KeySizeUser};
    use digest::{FixedOutput, HashMarker, MacMarker, OutputSizeUser, Update};

    use super::*;

    /// SHA-256 implementing RustCrypto traits like `Digest`.
    pub struct Sha256(Digest);

    /// HMAC-SHA-256 implementing RustCrypto traits like `Mac`.
    pub struct HmacSha256(Hmac);

    impl HashMarker for Sha256 {}

    impl Default for Sha256 {
        fn default() -> Self {
            Self(Digest::new(Algorithm::Sha256).unwrap())
        }
    }

    impl Update for Sha256 {
        fn update(&mut self, data: &[u8]) {
            self.0.update(data);
        }
    }

    impl OutputSizeUser for Sha256 {
        type OutputSize = digest::consts::U32;
    }

    impl FixedOutput for Sha256 {
        fn finalize_into(self, out: &mut digest::Output<Self>) {
            self.0.finalize(out).unwrap();
        }
    }

    impl MacMarker for HmacSha256 {}

    impl KeySizeUser for HmacSha256 {
        type KeySize = digest::consts::U64;
    }

    impl KeyInit for HmacSha256 {
        fn new(key: &digest::Key<Self>) -> Self {
            Self::new_from_slice(key).unwrap()
        }

        fn new_from_slice(key: &[u8]) -> Result<Self, crypto_common::InvalidLength> {
            Ok(Self(Hmac::new(Algorithm::Sha256, key).unwrap()))
        }
    }

    impl Update for HmacSha256 {
        fn update(&mut self, data: &[u8]) {
            self.0.update(data);
        }
    }

    impl OutputSizeUser for HmacSha256 {
        type OutputSize = digest::consts::U32;
    }

    impl FixedOutput for HmacSha256 {
        fn finalize_into(self, out: &mut digest::Output<Self>) {
            self.0.finalize(out).unwrap()
        }
    }
}
