// Copyright 2022 Google LLC
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

//! Cryptography interface.

#[cfg(feature = "internal-api-crypto-hash")]
use crypto_common::BlockSizeUser;
#[cfg(any(feature = "internal-api-crypto-hash", feature = "internal-api-crypto-hmac"))]
use crypto_common::Output;
#[cfg(feature = "internal-api-crypto-hmac")]
use crypto_common::{InvalidLength, KeyInit};
#[cfg(feature = "internal-api-crypto-hash")]
use digest::HashMarker;
#[cfg(feature = "internal-api-crypto-hmac")]
use digest::MacMarker;
#[cfg(any(feature = "internal-api-crypto-hash", feature = "internal-api-crypto-hmac"))]
use digest::{FixedOutput, Update};
#[cfg(feature = "internal-api-crypto-hmac")]
use wasefire_error::Code;

#[cfg(any(feature = "internal-api-crypto-hash", feature = "internal-api-crypto-hmac"))]
use crate::{Error, Support};

#[cfg(feature = "internal-api-crypto-aead")]
pub mod aead;
#[cfg(feature = "internal-api-crypto-cbc")]
pub mod cbc;
#[cfg(feature = "internal-api-crypto-ecc")]
pub mod ecc;

/// Cryptography interface.
pub trait Api: Send {
    /// AES-128-CCM interface.
    #[cfg(feature = "api-crypto-aes128-ccm")]
    type Aes128Ccm: aead::Api<typenum::U16, typenum::U13, Tag = typenum::U4>;

    /// AES-256-CBC interface.
    #[cfg(feature = "api-crypto-aes256-cbc")]
    type Aes256Cbc: cbc::Api<typenum::U32, typenum::U16>;

    /// AES-256-GCM interface.
    #[cfg(feature = "api-crypto-aes256-gcm")]
    type Aes256Gcm: aead::Api<typenum::U32, typenum::U12>;

    /// HMAC-SHA-256 interface.
    #[cfg(feature = "api-crypto-hmac-sha256")]
    type HmacSha256: Hmac<KeySize = typenum::U64, OutputSize = typenum::U32>;

    /// HMAC-SHA-384 interface.
    #[cfg(feature = "api-crypto-hmac-sha384")]
    type HmacSha384: Hmac<KeySize = typenum::U128, OutputSize = typenum::U48>;

    /// P-256 interface.
    #[cfg(feature = "api-crypto-p256")]
    type P256: ecc::Api<typenum::U32>;

    /// P-384 interface.
    #[cfg(feature = "api-crypto-p384")]
    type P384: ecc::Api<typenum::U48>;

    /// SHA-256 interface.
    #[cfg(feature = "api-crypto-sha256")]
    type Sha256: Hash<BlockSize = typenum::U64, OutputSize = typenum::U32>;

    /// SHA-384 interface.
    #[cfg(feature = "api-crypto-sha384")]
    type Sha384: Hash<BlockSize = typenum::U128, OutputSize = typenum::U48>;
}

/// Hash interface.
#[cfg(feature = "internal-api-crypto-hash")]
pub trait Hash:
    Support<bool> + Send + Default + BlockSizeUser + Update + FixedOutput + HashMarker + WithError
{
}
/// HMAC interface.
#[cfg(feature = "internal-api-crypto-hmac")]
pub trait Hmac:
    Support<bool> + Send + KeyInit + Update + FixedOutput + MacMarker + WithError
{
}

#[cfg(feature = "internal-api-crypto-hash")]
impl<T> Hash for T where T: Support<bool>
        + Send
        + Default
        + BlockSizeUser
        + Update
        + FixedOutput
        + HashMarker
        + WithError
{
}
#[cfg(feature = "internal-api-crypto-hmac")]
impl<T> Hmac for T where T: Support<bool> + Send + KeyInit + Update + FixedOutput + MacMarker + WithError
{}

/// Adds error support to operations with an infallible signature.
#[cfg(any(feature = "internal-api-crypto-hash", feature = "internal-api-crypto-hmac"))]
pub trait WithError {
    /// Executes a seemingly infallible operation with error support.
    ///
    /// The closure may actually call multiple seemingly infaillible operations. Each such call
    /// should support running after a previous one failed. This funtion returns an error if any
    /// such call failed.
    fn with_error<T>(operation: impl FnOnce() -> T) -> Result<T, Error>;
}

/// Helper trait for infaillible operations.
#[cfg(any(feature = "internal-api-crypto-hash", feature = "internal-api-crypto-hmac"))]
pub trait NoError {}

#[cfg(any(feature = "internal-api-crypto-hash", feature = "internal-api-crypto-hmac"))]
impl<T: NoError> WithError for T {
    fn with_error<R>(operation: impl FnOnce() -> R) -> Result<R, Error> {
        Ok(operation())
    }
}

/// AES-128-CCM interface.
#[cfg(feature = "api-crypto-aes128-ccm")]
pub type Aes128Ccm<B> = <super::Crypto<B> as Api>::Aes128Ccm;

/// AES-256-CBC interface.
#[cfg(feature = "api-crypto-aes256-cbc")]
pub type Aes256Cbc<B> = <super::Crypto<B> as Api>::Aes256Cbc;

/// AES-256-GCM interface.
#[cfg(feature = "api-crypto-aes256-gcm")]
pub type Aes256Gcm<B> = <super::Crypto<B> as Api>::Aes256Gcm;

/// HMAC-SHA-256 interface.
#[cfg(feature = "api-crypto-hmac-sha256")]
pub type HmacSha256<B> = <super::Crypto<B> as Api>::HmacSha256;

/// HMAC-SHA-384 interface.
#[cfg(feature = "api-crypto-hmac-sha384")]
pub type HmacSha384<B> = <super::Crypto<B> as Api>::HmacSha384;

/// P-256 interface.
#[cfg(feature = "api-crypto-p256")]
pub type P256<B> = <super::Crypto<B> as Api>::P256;

/// P-384 interface.
#[cfg(feature = "api-crypto-p384")]
pub type P384<B> = <super::Crypto<B> as Api>::P384;

/// SHA-256 interface.
#[cfg(feature = "api-crypto-sha256")]
pub type Sha256<B> = <super::Crypto<B> as Api>::Sha256;

/// SHA-384 interface.
#[cfg(feature = "api-crypto-sha384")]
pub type Sha384<B> = <super::Crypto<B> as Api>::Sha384;

/// AES-128-CCM interface.
#[cfg(feature = "software-crypto-aes128-ccm")]
pub type SoftwareAes128Ccm = ccm::Ccm<aes::Aes128, typenum::U4, typenum::U13>;

/// AES-256-CBC interface.
#[cfg(feature = "software-crypto-aes256-cbc")]
pub type SoftwareAes256Cbc = cbc::Software<aes::Aes256>;

/// AES-256-GCM interface.
#[cfg(feature = "software-crypto-aes256-gcm")]
pub type SoftwareAes256Gcm = aes_gcm::Aes256Gcm;

/// HMAC-SHA-256 interface.
#[cfg(feature = "software-crypto-hmac-sha256")]
pub type SoftwareHmacSha256<T> = hmac::SimpleHmac<<T as Api>::Sha256>;

/// HMAC-SHA-384 interface.
#[cfg(feature = "software-crypto-hmac-sha384")]
pub type SoftwareHmacSha384<T> = hmac::SimpleHmac<<T as Api>::Sha384>;

/// P-256 interface.
#[cfg(feature = "software-crypto-p256")]
pub type SoftwareP256<T> = ecc::Software<p256::NistP256, <T as Api>::Sha256>;

/// P-384 interface.
#[cfg(feature = "software-crypto-p384")]
pub type SoftwareP384<T> = ecc::Software<p384::NistP384, <T as Api>::Sha384>;

/// SHA-256 interface.
#[cfg(feature = "software-crypto-sha256")]
pub type SoftwareSha256 = sha2::Sha256;

/// SHA-384 interface.
#[cfg(feature = "software-crypto-sha384")]
pub type SoftwareSha384 = sha2::Sha384;

#[cfg(feature = "software-crypto-sha256")]
impl crate::Supported for sha2::Sha256 {}

#[cfg(feature = "software-crypto-sha384")]
impl crate::Supported for sha2::Sha384 {}

#[cfg(feature = "internal-software-crypto-hmac")]
impl<D> Support<bool> for hmac::SimpleHmac<D>
where D: Support<bool> + Default + BlockSizeUser + Update + FixedOutput + HashMarker + WithError
{
    const SUPPORT: bool = D::SUPPORT;
}

#[cfg(feature = "software-crypto-sha256")]
impl NoError for sha2::Sha256 {}

#[cfg(feature = "software-crypto-sha384")]
impl NoError for sha2::Sha384 {}

#[cfg(feature = "internal-software-crypto-hmac")]
impl<D> WithError for hmac::SimpleHmac<D>
where D: Support<bool> + Default + BlockSizeUser + Update + FixedOutput + HashMarker + WithError
{
    fn with_error<T>(operation: impl FnOnce() -> T) -> Result<T, Error> {
        D::with_error(operation)
    }
}

/// Hash wrapper with error support.
#[cfg(feature = "internal-api-crypto-hash")]
pub struct HashApi<T: Hash>(T);

/// HMAC wrapper with error support.
#[cfg(feature = "internal-api-crypto-hmac")]
pub struct HmacApi<T: Hmac>(T);

#[cfg(feature = "internal-api-crypto-hash")]
impl<T: Hash> HashApi<T> {
    /// Creates a hash wrapper.
    pub fn new() -> Result<Self, Error> {
        T::with_error(|| T::default()).map(HashApi)
    }

    /// Updates the hash with the provided data.
    pub fn update(&mut self, data: &[u8]) -> Result<(), Error> {
        T::with_error(|| self.0.update(data))
    }

    /// Finalizes the hash to the provided output.
    pub fn finalize_into(self, out: &mut Output<T>) -> Result<(), Error> {
        T::with_error(|| self.0.finalize_into(out))
    }
}

#[cfg(feature = "internal-api-crypto-hmac")]
impl<T: Hmac> HmacApi<T> {
    /// Creates an HMAC wrapper.
    pub fn new(key: &[u8]) -> Result<Self, Error> {
        match T::with_error(|| T::new_from_slice(key)) {
            Ok(Ok(x)) => Ok(HmacApi(x)),
            Ok(Err(InvalidLength)) => Err(Error::user(Code::InvalidLength)),
            Err(e) => Err(e),
        }
    }

    /// Updates the HMAC with the provided data.
    pub fn update(&mut self, data: &[u8]) -> Result<(), Error> {
        T::with_error(|| self.0.update(data))
    }

    /// Finalizes the HMAC to the provided output.
    pub fn finalize_into(self, out: &mut Output<T>) -> Result<(), Error> {
        T::with_error(|| self.0.finalize_into(out))
    }
}
