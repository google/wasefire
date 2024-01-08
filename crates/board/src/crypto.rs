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
use crypto_common::{BlockSizeUser, Reset};
#[cfg(feature = "internal-api-crypto-hmac")]
use crypto_common::{Key, KeyInit, KeySizeUser};
#[cfg(any(feature = "internal-api-crypto-hash", feature = "internal-api-crypto-hmac"))]
use crypto_common::{Output, OutputSizeUser};
#[cfg(feature = "internal-api-crypto-hmac")]
use digest::MacMarker;
#[cfg(any(feature = "internal-api-crypto-hash", feature = "internal-api-crypto-hmac"))]
use digest::{FixedOutput, Update};
#[cfg(feature = "internal-api-crypto-hash")]
use digest::{FixedOutputReset, HashMarker};
#[cfg(any(feature = "internal-api-crypto-hash", feature = "internal-api-crypto-hmac"))]
use generic_array::ArrayLength;

#[cfg(any(feature = "internal-api-crypto-hash", feature = "internal-api-crypto-hmac"))]
use crate::Support;
use crate::Unsupported;

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

#[cfg(feature = "internal-api-crypto-hash")]
pub struct UnsupportedHash<Block: ArrayLength<u8>, Output: ArrayLength<u8>> {
    _never: !,
    _block: Block,
    _output: Output,
}
#[cfg(feature = "internal-api-crypto-hmac")]
pub struct UnsupportedHmac<Key: ArrayLength<u8>, Output: ArrayLength<u8>> {
    _never: !,
    _key: Key,
    _output: Output,
}

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

pub struct UnsupportedCrypto<T: Api>(T);

macro_rules! software {
    (#[cfg(feature = $feature:literal)] type $Name:ident = $impl:ty | $Unsupported:ty;) => {
        #[cfg(feature = $feature)]
        type $Name = $impl;
        #[cfg(not(feature = $feature))]
        type $Name = $Unsupported;
    };
}

impl<T: Api> Api for UnsupportedCrypto<T> {
    #[cfg(feature = "api-crypto-aes128-ccm")]
    software! {
        #[cfg(feature = "software-crypto-aes128-ccm")]
        type Aes128Ccm = ccm::Ccm<aes::Aes128, typenum::U4, typenum::U13>
                       | aead::Unsupported<typenum::U4>;
    }
    #[cfg(feature = "api-crypto-aes256-gcm")]
    software! {
        #[cfg(feature = "software-crypto-aes256-gcm")]
        type Aes256Gcm = aes_gcm::Aes256Gcm | aead::Unsupported<typenum::U16>;
    }

    #[cfg(feature = "api-crypto-hmac-sha256")]
    software! {
        #[cfg(feature = "software-crypto-hmac-sha256")]
        type HmacSha256 = hmac::SimpleHmac<T::Sha256>
                        | UnsupportedHmac<typenum::U64, typenum::U32>;
    }
    #[cfg(feature = "api-crypto-hmac-sha384")]
    software! {
        #[cfg(feature = "software-crypto-hmac-sha384")]
        type HmacSha384 = hmac::SimpleHmac<T::Sha384>
                        | UnsupportedHmac<typenum::U128, typenum::U48>;
    }

    #[cfg(feature = "api-crypto-p256")]
    software! {
        #[cfg(feature = "software-crypto-p256")]
        type P256 = ecc::Software<p256::NistP256, T::Sha256> | Unsupported;
    }
    #[cfg(feature = "api-crypto-p384")]
    software! {
        #[cfg(feature = "software-crypto-p384")]
        type P384 = ecc::Software<p384::NistP384, T::Sha384> | Unsupported;
    }

    #[cfg(feature = "api-crypto-sha256")]
    software! {
        #[cfg(feature = "software-crypto-sha256")]
        type Sha256 = sha2::Sha256 | UnsupportedHash<typenum::U64, typenum::U32>;
    }
    #[cfg(feature = "api-crypto-sha384")]
    software! {
        #[cfg(feature = "software-crypto-sha384")]
        type Sha384 = sha2::Sha384 | UnsupportedHash<typenum::U128, typenum::U48>;
    }
}

impl Api for Unsupported {
    #[cfg(feature = "api-crypto-aes128-ccm")]
    type Aes128Ccm = <UnsupportedCrypto<Self> as Api>::Aes128Ccm;
    #[cfg(feature = "api-crypto-aes256-gcm")]
    type Aes256Gcm = <UnsupportedCrypto<Self> as Api>::Aes256Gcm;
    #[cfg(feature = "api-crypto-hmac-sha256")]
    type HmacSha256 = <UnsupportedCrypto<Self> as Api>::HmacSha256;
    #[cfg(feature = "api-crypto-hmac-sha384")]
    type HmacSha384 = <UnsupportedCrypto<Self> as Api>::HmacSha384;
    #[cfg(feature = "api-crypto-p256")]
    type P256 = <UnsupportedCrypto<Self> as Api>::P256;
    #[cfg(feature = "api-crypto-p384")]
    type P384 = <UnsupportedCrypto<Self> as Api>::P384;
    #[cfg(feature = "api-crypto-sha256")]
    type Sha256 = <UnsupportedCrypto<Self> as Api>::Sha256;
    #[cfg(feature = "api-crypto-sha384")]
    type Sha384 = <UnsupportedCrypto<Self> as Api>::Sha384;
}

#[cfg(feature = "internal-api-crypto-hash")]
impl<B, O> BlockSizeUser for UnsupportedHash<B, O>
where
    B: ArrayLength<u8>,
    O: ArrayLength<u8>,
{
    type BlockSize = B;
}

#[cfg(feature = "internal-api-crypto-hash")]
impl<B, O> OutputSizeUser for UnsupportedHash<B, O>
where
    B: ArrayLength<u8>,
    O: ArrayLength<u8>,
{
    type OutputSize = O;
}

#[cfg(feature = "internal-api-crypto-hash")]
impl<B, O> HashMarker for UnsupportedHash<B, O>
where
    B: ArrayLength<u8>,
    O: ArrayLength<u8>,
{
}

#[cfg(feature = "internal-api-crypto-hash")]
impl<B, O> Default for UnsupportedHash<B, O>
where
    B: ArrayLength<u8>,
    O: ArrayLength<u8>,
{
    fn default() -> Self {
        unreachable!()
    }
}

#[cfg(feature = "internal-api-crypto-hash")]
impl<B, O> Update for UnsupportedHash<B, O>
where
    B: ArrayLength<u8>,
    O: ArrayLength<u8>,
{
    fn update(&mut self, _: &[u8]) {
        unreachable!()
    }
}

#[cfg(feature = "internal-api-crypto-hash")]
impl<B, O> FixedOutput for UnsupportedHash<B, O>
where
    B: ArrayLength<u8>,
    O: ArrayLength<u8>,
{
    fn finalize_into(self, _: &mut Output<Self>) {
        unreachable!()
    }
}

#[cfg(feature = "internal-api-crypto-hash")]
impl<B, O> FixedOutputReset for UnsupportedHash<B, O>
where
    B: ArrayLength<u8>,
    O: ArrayLength<u8>,
{
    fn finalize_into_reset(&mut self, _: &mut Output<Self>) {
        unreachable!()
    }
}

#[cfg(feature = "internal-api-crypto-hash")]
impl<B, O> Reset for UnsupportedHash<B, O>
where
    B: ArrayLength<u8>,
    O: ArrayLength<u8>,
{
    fn reset(&mut self) {
        unreachable!()
    }
}

#[cfg(feature = "internal-api-crypto-hash")]
impl<B, O> Support<bool> for UnsupportedHash<B, O>
where
    B: ArrayLength<u8>,
    O: ArrayLength<u8>,
{
    const SUPPORT: bool = false;
}

#[cfg(feature = "internal-api-crypto-hmac")]
impl<K, O> KeySizeUser for UnsupportedHmac<K, O>
where
    K: ArrayLength<u8>,
    O: ArrayLength<u8>,
{
    type KeySize = K;
}

#[cfg(feature = "internal-api-crypto-hmac")]
impl<K, O> OutputSizeUser for UnsupportedHmac<K, O>
where
    K: ArrayLength<u8>,
    O: ArrayLength<u8>,
{
    type OutputSize = O;
}

#[cfg(feature = "internal-api-crypto-hmac")]
impl<K, O> MacMarker for UnsupportedHmac<K, O>
where
    K: ArrayLength<u8>,
    O: ArrayLength<u8>,
{
}

#[cfg(feature = "internal-api-crypto-hmac")]
impl<K, O> KeyInit for UnsupportedHmac<K, O>
where
    K: ArrayLength<u8>,
    O: ArrayLength<u8>,
{
    fn new(_: &Key<Self>) -> Self {
        unreachable!()
    }
}

#[cfg(feature = "internal-api-crypto-hmac")]
impl<K, O> Update for UnsupportedHmac<K, O>
where
    K: ArrayLength<u8>,
    O: ArrayLength<u8>,
{
    fn update(&mut self, _: &[u8]) {
        unreachable!()
    }
}

#[cfg(feature = "internal-api-crypto-hmac")]
impl<K, O> FixedOutput for UnsupportedHmac<K, O>
where
    K: ArrayLength<u8>,
    O: ArrayLength<u8>,
{
    fn finalize_into(self, _: &mut Output<Self>) {
        unreachable!()
    }
}

#[cfg(feature = "internal-api-crypto-hmac")]
impl<K, O> Support<bool> for UnsupportedHmac<K, O>
where
    K: ArrayLength<u8>,
    O: ArrayLength<u8>,
{
    const SUPPORT: bool = false;
}

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
