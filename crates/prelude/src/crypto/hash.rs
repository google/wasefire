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
use wasefire_error::Code;

pub use self::api::Algorithm;
use crate::{Error, convert, convert_bool, convert_unit};

/// Hashing context.
#[cfg(feature = "api-crypto-hash")]
pub struct Digest {
    /// The hashing context identifier.
    ///
    /// Finalized when -1 (as usize). This is used to know whether Drop should finalize.
    id: usize,

    /// The digest length in bytes.
    len: usize,
}

#[cfg(feature = "api-crypto-hash")]
impl Digest {
    /// Creates a new hashing context for the specified algorithm.
    pub fn new(algorithm: Algorithm) -> Result<Self, Error> {
        if !is_supported(algorithm) {
            return Err(Error::world(Code::NotImplemented));
        }
        let params = api::initialize::Params { algorithm: algorithm as usize };
        let id = convert(unsafe { api::initialize(params) })?;
        let len = algorithm.digest_len();
        Ok(Self { id, len })
    }

    /// Updates the hashing context with the provided data.
    pub fn update(&mut self, data: &[u8]) {
        let params = api::update::Params { id: self.id, data: data.as_ptr(), length: data.len() };
        convert_unit(unsafe { api::update(params) }).unwrap();
    }

    /// Finalizes the hashing context and writes the associated digest.
    pub fn finalize(mut self, digest: &mut [u8]) -> Result<(), Error> {
        if digest.len() != self.len {
            return Err(Error::user(0));
        }
        let params = api::finalize::Params { id: self.id, digest: digest.as_mut_ptr() };
        self.id = usize::MAX;
        convert_unit(unsafe { api::finalize(params) })
    }

    /// Writes the hash of the data for the given algorithm in the digest.
    pub fn digest(algorithm: Algorithm, data: &[u8], digest: &mut [u8]) -> Result<(), Error> {
        let mut context = Self::new(algorithm)?;
        context.update(data);
        context.finalize(digest)
    }
}

#[cfg(feature = "api-crypto-hash")]
impl Drop for Digest {
    fn drop(&mut self) {
        if self.id == usize::MAX {
            // Already finalized.
            return;
        }
        let params = api::finalize::Params { id: self.id, digest: core::ptr::null_mut() };
        convert_unit(unsafe { api::finalize(params) }).unwrap();
    }
}

/// Hmac context.
#[cfg(feature = "api-crypto-hmac")]
pub struct Hmac {
    /// The hmac context identifier.
    ///
    /// Finalized when -1 (as usize). This is used to know whether Drop should finalize.
    id: usize,

    /// The hmac length in bytes.
    len: usize,
}

#[cfg(feature = "api-crypto-hmac")]
impl Hmac {
    /// Creates a new hmac context for the specified algorithm.
    pub fn new(algorithm: Algorithm, key: &[u8]) -> Result<Self, Error> {
        if !is_hmac_supported(algorithm) {
            return Err(Error::world(Code::NotImplemented));
        }
        let params = api::hmac_initialize::Params {
            algorithm: algorithm as usize,
            key: key.as_ptr(),
            key_len: key.len(),
        };
        let id = convert(unsafe { api::hmac_initialize(params) })?;
        let len = algorithm.digest_len();
        Ok(Self { id, len })
    }

    /// Updates the hmac context with the provided data.
    pub fn update(&mut self, data: &[u8]) {
        let params =
            api::hmac_update::Params { id: self.id, data: data.as_ptr(), length: data.len() };
        convert_unit(unsafe { api::hmac_update(params) }).unwrap();
    }

    /// Finalizes the hmac context and writes the associated hmac.
    pub fn finalize(mut self, hmac: &mut [u8]) -> Result<(), Error> {
        if hmac.len() != self.len {
            return Err(Error::user(0));
        }
        let params = api::hmac_finalize::Params { id: self.id, hmac: hmac.as_mut_ptr() };
        self.id = usize::MAX;
        convert_unit(unsafe { api::hmac_finalize(params) })
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

#[cfg(feature = "api-crypto-hmac")]
impl Drop for Hmac {
    fn drop(&mut self) {
        if self.id == usize::MAX {
            // Already finalized.
            return;
        }
        let params = api::hmac_finalize::Params { id: self.id, hmac: core::ptr::null_mut() };
        convert_unit(unsafe { api::hmac_finalize(params) }).unwrap();
    }
}

/// Returns the SHA-256 of the provided data.
#[cfg(feature = "api-crypto-hash")]
pub fn sha256(data: &[u8]) -> Result<[u8; 32], Error> {
    let mut digest = [0; 32];
    Digest::digest(Algorithm::Sha256, data, &mut digest)?;
    Ok(digest)
}

/// Returns the HMAC-SHA-256 of the provided data.
#[cfg(feature = "api-crypto-hmac")]
pub fn hmac_sha256(key: &[u8], data: &[u8]) -> Result<[u8; 32], Error> {
    let mut hmac = [0; 32];
    Hmac::hmac(Algorithm::Sha256, key, data, &mut hmac)?;
    Ok(hmac)
}

/// Derives a key according to HKDF.
#[cfg(feature = "api-crypto-hkdf")]
pub fn hkdf(
    algorithm: Algorithm, salt: Option<&[u8]>, ikm: &[u8], info: &[u8], okm: &mut [u8],
) -> Result<(), Error> {
    let mut prk = alloc::vec![0; algorithm.digest_len()];
    hkdf_extract(algorithm, salt, ikm, &mut prk)?;
    hkdf_expand(algorithm, &prk, info, okm)
}

/// The extract operation of HKDF.
#[cfg(feature = "api-crypto-hkdf")]
pub fn hkdf_extract(
    algorithm: Algorithm, salt: Option<&[u8]>, ikm: &[u8], prk: &mut [u8],
) -> Result<(), Error> {
    use alloc::borrow::Cow;
    let salt = match salt {
        Some(x) => Cow::Borrowed(x),
        None => Cow::Owned(alloc::vec![0; algorithm.digest_len()]),
    };
    Hmac::hmac(algorithm, &salt, ikm, prk)
}

/// The expand operation of HKDF.
#[cfg(feature = "api-crypto-hkdf")]
pub fn hkdf_expand(
    algorithm: Algorithm, prk: &[u8], info: &[u8], okm: &mut [u8],
) -> Result<(), Error> {
    let params = api::hkdf_expand::Params {
        algorithm: algorithm as usize,
        prk: prk.as_ptr(),
        prk_len: prk.len(),
        info: info.as_ptr(),
        info_len: info.len(),
        okm: okm.as_mut_ptr(),
        okm_len: okm.len(),
    };
    convert_unit(unsafe { api::hkdf_expand(params) })
}

/// Whether a hash algorithm is supported.
#[cfg(feature = "api-crypto-hash")]
pub fn is_supported(algorithm: Algorithm) -> bool {
    let params = api::is_supported::Params { algorithm: algorithm as usize };
    convert_bool(unsafe { api::is_supported(params) }).unwrap_or(false)
}

/// Whether a hash algorithm is supported for HMAC.
#[cfg(feature = "api-crypto-hmac")]
pub fn is_hmac_supported(algorithm: Algorithm) -> bool {
    let params = api::is_hmac_supported::Params { algorithm: algorithm as usize };
    convert_bool(unsafe { api::is_hmac_supported(params) }).unwrap_or(false)
}

/// Whether a hash algorithm is supported for HKDF.
#[cfg(feature = "api-crypto-hkdf")]
pub fn is_hkdf_supported(algorithm: Algorithm) -> bool {
    let params = api::is_hkdf_supported::Params { algorithm: algorithm as usize };
    convert_bool(unsafe { api::is_hkdf_supported(params) }).unwrap_or(false)
}

#[cfg(feature = "rust-crypto")]
mod rust_crypto {
    #[cfg(feature = "api-crypto-hmac")]
    use crypto_common::{KeyInit, KeySizeUser};
    #[cfg(feature = "api-crypto-hash")]
    use digest::HashMarker;
    #[cfg(feature = "api-crypto-hmac")]
    use digest::MacMarker;
    use digest::{FixedOutput, OutputSizeUser, Update};

    use super::*;

    /// SHA-256 implementing RustCrypto traits like `Digest`.
    #[cfg(feature = "api-crypto-hash")]
    pub struct Sha256(Digest);

    /// SHA-384 implementing RustCrypto traits like `Digest`.
    #[cfg(feature = "api-crypto-hash")]
    pub struct Sha384(Digest);

    /// HMAC-SHA-256 implementing RustCrypto traits like `Mac`.
    #[cfg(feature = "api-crypto-hmac")]
    pub struct HmacSha256(Hmac);

    /// HMAC-SHA-384 implementing RustCrypto traits like `Mac`.
    #[cfg(feature = "api-crypto-hmac")]
    pub struct HmacSha384(Hmac);

    #[cfg(feature = "api-crypto-hash")]
    impl HashMarker for Sha256 {}

    #[cfg(feature = "api-crypto-hash")]
    impl HashMarker for Sha384 {}

    #[cfg(feature = "api-crypto-hash")]
    impl Default for Sha256 {
        fn default() -> Self {
            Self(Digest::new(Algorithm::Sha256).unwrap())
        }
    }

    #[cfg(feature = "api-crypto-hash")]
    impl Default for Sha384 {
        fn default() -> Self {
            Self(Digest::new(Algorithm::Sha384).unwrap())
        }
    }

    #[cfg(feature = "api-crypto-hash")]
    impl Update for Sha256 {
        fn update(&mut self, data: &[u8]) {
            self.0.update(data);
        }
    }

    #[cfg(feature = "api-crypto-hash")]
    impl Update for Sha384 {
        fn update(&mut self, data: &[u8]) {
            self.0.update(data);
        }
    }

    #[cfg(feature = "api-crypto-hash")]
    impl OutputSizeUser for Sha256 {
        type OutputSize = digest::consts::U32;
    }

    #[cfg(feature = "api-crypto-hash")]
    impl OutputSizeUser for Sha384 {
        type OutputSize = digest::consts::U48;
    }

    #[cfg(feature = "api-crypto-hash")]
    impl FixedOutput for Sha256 {
        fn finalize_into(self, out: &mut digest::Output<Self>) {
            self.0.finalize(out).unwrap();
        }
    }

    #[cfg(feature = "api-crypto-hash")]
    impl FixedOutput for Sha384 {
        fn finalize_into(self, out: &mut digest::Output<Self>) {
            self.0.finalize(out).unwrap();
        }
    }

    #[cfg(feature = "api-crypto-hmac")]
    impl MacMarker for HmacSha256 {}

    #[cfg(feature = "api-crypto-hmac")]
    impl MacMarker for HmacSha384 {}

    #[cfg(feature = "api-crypto-hmac")]
    impl KeySizeUser for HmacSha256 {
        type KeySize = digest::consts::U64;
    }

    #[cfg(feature = "api-crypto-hmac")]
    impl KeySizeUser for HmacSha384 {
        type KeySize = digest::consts::U128;
    }

    #[cfg(feature = "api-crypto-hmac")]
    impl KeyInit for HmacSha256 {
        fn new(key: &digest::Key<Self>) -> Self {
            Self::new_from_slice(key).unwrap()
        }

        fn new_from_slice(key: &[u8]) -> Result<Self, crypto_common::InvalidLength> {
            match Hmac::new(Algorithm::Sha256, key) {
                Err(e) if e == Error::user(Code::InvalidLength) => {
                    Err(crypto_common::InvalidLength)
                }
                x => Ok(Self(x.unwrap())),
            }
        }
    }

    #[cfg(feature = "api-crypto-hmac")]
    impl KeyInit for HmacSha384 {
        fn new(key: &digest::Key<Self>) -> Self {
            Self::new_from_slice(key).unwrap()
        }

        fn new_from_slice(key: &[u8]) -> Result<Self, crypto_common::InvalidLength> {
            Ok(Self(Hmac::new(Algorithm::Sha384, key).unwrap()))
        }
    }

    #[cfg(feature = "api-crypto-hmac")]
    impl Update for HmacSha256 {
        fn update(&mut self, data: &[u8]) {
            self.0.update(data);
        }
    }

    #[cfg(feature = "api-crypto-hmac")]
    impl Update for HmacSha384 {
        fn update(&mut self, data: &[u8]) {
            self.0.update(data);
        }
    }

    #[cfg(feature = "api-crypto-hmac")]
    impl OutputSizeUser for HmacSha256 {
        type OutputSize = digest::consts::U32;
    }

    #[cfg(feature = "api-crypto-hmac")]
    impl OutputSizeUser for HmacSha384 {
        type OutputSize = digest::consts::U48;
    }

    #[cfg(feature = "api-crypto-hmac")]
    impl FixedOutput for HmacSha256 {
        fn finalize_into(self, out: &mut digest::Output<Self>) {
            self.0.finalize(out).unwrap()
        }
    }

    #[cfg(feature = "api-crypto-hmac")]
    impl FixedOutput for HmacSha384 {
        fn finalize_into(self, out: &mut digest::Output<Self>) {
            self.0.finalize(out).unwrap()
        }
    }
}
