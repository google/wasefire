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
#[cfg(feature = "internal-api-crypto-hmac")]
use crypto_common::KeyInit;
#[cfg(any(feature = "internal-api-crypto-hash", feature = "internal-api-crypto-hmac"))]
use digest::Update;
#[cfg(feature = "internal-api-crypto-hmac")]
use digest::{FixedOutput, MacMarker};
#[cfg(feature = "internal-api-crypto-hash")]
use digest::{FixedOutputReset, HashMarker};

#[cfg(any(feature = "internal-api-crypto-hash", feature = "internal-api-crypto-hmac"))]
use crate::Support;

#[cfg(feature = "internal-api-crypto-aead")]
pub mod aead;
#[cfg(feature = "internal-api-crypto-ecc")]
pub mod ecc;

/// Cryptography interface.
pub trait Api: Send {
    #[cfg(feature = "api-crypto-aes128-ccm")]
    type Aes128Ccm: aead::Api<typenum::U16, typenum::U13, Tag = typenum::U4>;
    #[cfg(feature = "api-crypto-aes256-gcm")]
    type Aes256Gcm: aead::Api<typenum::U32, typenum::U12>;

    #[cfg(feature = "api-crypto-hmac-sha256")]
    type HmacSha256: Hmac<KeySize = typenum::U64, OutputSize = typenum::U32>;
    #[cfg(feature = "api-crypto-hmac-sha384")]
    type HmacSha384: Hmac<KeySize = typenum::U128, OutputSize = typenum::U48>;

    #[cfg(feature = "api-crypto-p256")]
    type P256: ecc::Api<typenum::U32>;
    #[cfg(feature = "api-crypto-p384")]
    type P384: ecc::Api<typenum::U48>;

    #[cfg(feature = "api-crypto-sha256")]
    type Sha256: Hash<BlockSize = typenum::U64, OutputSize = typenum::U32>;
    #[cfg(feature = "api-crypto-sha384")]
    type Sha384: Hash<BlockSize = typenum::U128, OutputSize = typenum::U48>;
}

#[cfg(feature = "internal-api-crypto-hash")]
pub trait Hash:
    Support<bool> + Send + Default + BlockSizeUser + Update + FixedOutputReset + HashMarker
{
}
#[cfg(feature = "internal-api-crypto-hmac")]
pub trait Hmac: Support<bool> + Send + KeyInit + Update + FixedOutput + MacMarker {}

#[cfg(feature = "internal-api-crypto-hash")]
impl<
        T: Support<bool> + Send + Default + BlockSizeUser + Update + FixedOutputReset + HashMarker,
    > Hash for T
{
}
#[cfg(feature = "internal-api-crypto-hmac")]
impl<T: Support<bool> + Send + KeyInit + Update + FixedOutput + MacMarker> Hmac for T {}

#[cfg(feature = "api-crypto-aes128-ccm")]
pub type Aes128Ccm<B> = <super::Crypto<B> as Api>::Aes128Ccm;
#[cfg(feature = "api-crypto-aes256-gcm")]
pub type Aes256Gcm<B> = <super::Crypto<B> as Api>::Aes256Gcm;
#[cfg(feature = "api-crypto-hmac-sha256")]
pub type HmacSha256<B> = <super::Crypto<B> as Api>::HmacSha256;
#[cfg(feature = "api-crypto-hmac-sha384")]
pub type HmacSha384<B> = <super::Crypto<B> as Api>::HmacSha384;
#[cfg(feature = "api-crypto-p256")]
pub type P256<B> = <super::Crypto<B> as Api>::P256;
#[cfg(feature = "api-crypto-p384")]
pub type P384<B> = <super::Crypto<B> as Api>::P384;
#[cfg(feature = "api-crypto-sha256")]
pub type Sha256<B> = <super::Crypto<B> as Api>::Sha256;
#[cfg(feature = "api-crypto-sha384")]
pub type Sha384<B> = <super::Crypto<B> as Api>::Sha384;

#[cfg(feature = "software-crypto-aes128-ccm")]
pub type SoftwareAes128Ccm = ccm::Ccm<aes::Aes128, typenum::U4, typenum::U13>;
#[cfg(feature = "software-crypto-aes256-gcm")]
pub type SoftwareAes256Gcm = aes_gcm::Aes256Gcm;
#[cfg(feature = "software-crypto-hmac-sha256")]
pub type SoftwareHmacSha256<T> = hmac::SimpleHmac<<T as Api>::Sha256>;
#[cfg(feature = "software-crypto-hmac-sha384")]
pub type SoftwareHmacSha384<T> = hmac::SimpleHmac<<T as Api>::Sha384>;
#[cfg(feature = "software-crypto-p256")]
pub type SoftwareP256<T> = ecc::Software<p256::NistP256, <T as Api>::Sha256>;
#[cfg(feature = "software-crypto-p384")]
pub type SoftwareP384<T> = ecc::Software<p384::NistP384, <T as Api>::Sha384>;
#[cfg(feature = "software-crypto-sha256")]
pub type SoftwareSha256 = sha2::Sha256;
#[cfg(feature = "software-crypto-sha384")]
pub type SoftwareSha384 = sha2::Sha384;

#[cfg(feature = "software-crypto-sha256")]
impl crate::Supported for sha2::Sha256 {}

#[cfg(feature = "software-crypto-sha384")]
impl crate::Supported for sha2::Sha384 {}

#[cfg(feature = "internal-software-crypto-hmac")]
impl<D: Support<bool> + Default + BlockSizeUser + Update + FixedOutput + HashMarker> Support<bool>
    for hmac::SimpleHmac<D>
{
    const SUPPORT: bool = D::SUPPORT;
}
