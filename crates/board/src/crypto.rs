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

use crypto_common::{BlockSizeUser, Key, KeyInit, KeySizeUser, Output, OutputSizeUser, Reset};
use digest::{FixedOutput, FixedOutputReset, HashMarker, MacMarker, Update};
use generic_array::ArrayLength;
use typenum::{U12, U128, U13, U16, U32, U4, U48, U64};

use crate::{Support, Unsupported};

pub mod aead;
pub mod ecc;

/// Cryptography interface.
pub trait Api {
    type Aes128Ccm: aead::Api<U16, U13, Tag = U4>;
    type Aes256Gcm: aead::Api<U32, U12>;

    type HmacSha256: Support<bool> + Hmac<KeySize = U64, OutputSize = U32>;
    type HmacSha384: Support<bool> + Hmac<KeySize = U128, OutputSize = U48>;

    type P256: Support<bool> + ecc::Api<U32>;
    type P384: Support<bool> + ecc::Api<U48>;

    type Sha256: Support<bool> + Hash<BlockSize = U64, OutputSize = U32>;
    type Sha384: Support<bool> + Hash<BlockSize = U128, OutputSize = U48>;
}

pub trait Hash: Default + BlockSizeUser + Update + FixedOutputReset + HashMarker {}
pub trait Hmac: KeyInit + Update + FixedOutput + MacMarker {}

impl<T: Default + BlockSizeUser + Update + FixedOutputReset + HashMarker> Hash for T {}
impl<T: KeyInit + Update + FixedOutput + MacMarker> Hmac for T {}

pub struct UnsupportedHash<Block: ArrayLength<u8> + 'static, Output: ArrayLength<u8> + 'static> {
    _never: !,
    _block: Block,
    _output: Output,
}
pub struct UnsupportedHmac<Key: ArrayLength<u8> + 'static, Output: ArrayLength<u8> + 'static> {
    _never: !,
    _key: Key,
    _output: Output,
}

pub type Aes128Ccm<B> = <super::Crypto<B> as Api>::Aes128Ccm;
pub type Aes256Gcm<B> = <super::Crypto<B> as Api>::Aes256Gcm;
pub type HmacSha256<B> = <super::Crypto<B> as Api>::HmacSha256;
pub type HmacSha384<B> = <super::Crypto<B> as Api>::HmacSha384;
pub type P256<B> = <super::Crypto<B> as Api>::P256;
pub type P384<B> = <super::Crypto<B> as Api>::P384;
pub type Sha256<B> = <super::Crypto<B> as Api>::Sha256;
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
    software! {
        #[cfg(feature = "software-crypto-aes128-ccm")]
        type Aes128Ccm = ccm::Ccm<aes::Aes128, U4, U13> | aead::Unsupported<U4>;
    }
    software! {
        #[cfg(feature = "software-crypto-aes256-gcm")]
        type Aes256Gcm = aes_gcm::Aes256Gcm | aead::Unsupported<U16>;
    }

    software! {
        #[cfg(feature = "software-crypto-hmac-sha256")]
        type HmacSha256 = hmac::SimpleHmac<T::Sha256> | UnsupportedHmac<U64, U32>;
    }
    software! {
        #[cfg(feature = "software-crypto-hmac-sha384")]
        type HmacSha384 = hmac::SimpleHmac<T::Sha384> | UnsupportedHmac<U128, U48>;
    }

    software! {
        #[cfg(feature = "software-crypto-p256")]
        type P256 = ecc::Software<p256::NistP256, T::Sha256> | Unsupported;
    }
    software! {
        #[cfg(feature = "software-crypto-p384")]
        type P384 = ecc::Software<p384::NistP384, T::Sha384> | Unsupported;
    }

    software! {
        #[cfg(feature = "software-crypto-sha256")]
        type Sha256 = sha2::Sha256 | UnsupportedHash<U64, U32>;
    }
    software! {
        #[cfg(feature = "software-crypto-sha384")]
        type Sha384 = sha2::Sha384 | UnsupportedHash<U128, U48>;
    }
}

impl Api for Unsupported {
    type Aes128Ccm = <UnsupportedCrypto<Self> as Api>::Aes128Ccm;
    type Aes256Gcm = <UnsupportedCrypto<Self> as Api>::Aes256Gcm;
    type HmacSha256 = <UnsupportedCrypto<Self> as Api>::HmacSha256;
    type HmacSha384 = <UnsupportedCrypto<Self> as Api>::HmacSha384;
    type P256 = <UnsupportedCrypto<Self> as Api>::P256;
    type P384 = <UnsupportedCrypto<Self> as Api>::P384;
    type Sha256 = <UnsupportedCrypto<Self> as Api>::Sha256;
    type Sha384 = <UnsupportedCrypto<Self> as Api>::Sha384;
}

impl<B, O> BlockSizeUser for UnsupportedHash<B, O>
where
    B: ArrayLength<u8> + 'static,
    O: ArrayLength<u8> + 'static,
{
    type BlockSize = B;
}

impl<B, O> OutputSizeUser for UnsupportedHash<B, O>
where
    B: ArrayLength<u8> + 'static,
    O: ArrayLength<u8> + 'static,
{
    type OutputSize = O;
}

impl<B, O> HashMarker for UnsupportedHash<B, O>
where
    B: ArrayLength<u8> + 'static,
    O: ArrayLength<u8> + 'static,
{
}

impl<B, O> Default for UnsupportedHash<B, O>
where
    B: ArrayLength<u8> + 'static,
    O: ArrayLength<u8> + 'static,
{
    fn default() -> Self {
        unreachable!()
    }
}

impl<B, O> Update for UnsupportedHash<B, O>
where
    B: ArrayLength<u8> + 'static,
    O: ArrayLength<u8> + 'static,
{
    fn update(&mut self, _: &[u8]) {
        unreachable!()
    }
}

impl<B, O> FixedOutput for UnsupportedHash<B, O>
where
    B: ArrayLength<u8> + 'static,
    O: ArrayLength<u8> + 'static,
{
    fn finalize_into(self, _: &mut Output<Self>) {
        unreachable!()
    }
}

impl<B, O> FixedOutputReset for UnsupportedHash<B, O>
where
    B: ArrayLength<u8> + 'static,
    O: ArrayLength<u8> + 'static,
{
    fn finalize_into_reset(&mut self, _: &mut Output<Self>) {
        unreachable!()
    }
}

impl<B, O> Reset for UnsupportedHash<B, O>
where
    B: ArrayLength<u8> + 'static,
    O: ArrayLength<u8> + 'static,
{
    fn reset(&mut self) {
        unreachable!()
    }
}

impl<B, O> Support<bool> for UnsupportedHash<B, O>
where
    B: ArrayLength<u8> + 'static,
    O: ArrayLength<u8> + 'static,
{
    const SUPPORT: bool = false;
}

impl<K, O> KeySizeUser for UnsupportedHmac<K, O>
where
    K: ArrayLength<u8> + 'static,
    O: ArrayLength<u8> + 'static,
{
    type KeySize = K;
}

impl<K, O> OutputSizeUser for UnsupportedHmac<K, O>
where
    K: ArrayLength<u8> + 'static,
    O: ArrayLength<u8> + 'static,
{
    type OutputSize = O;
}

impl<K, O> MacMarker for UnsupportedHmac<K, O>
where
    K: ArrayLength<u8> + 'static,
    O: ArrayLength<u8> + 'static,
{
}

impl<K, O> KeyInit for UnsupportedHmac<K, O>
where
    K: ArrayLength<u8> + 'static,
    O: ArrayLength<u8> + 'static,
{
    fn new(_: &Key<Self>) -> Self {
        unreachable!()
    }
}

impl<K, O> Update for UnsupportedHmac<K, O>
where
    K: ArrayLength<u8> + 'static,
    O: ArrayLength<u8> + 'static,
{
    fn update(&mut self, _: &[u8]) {
        unreachable!()
    }
}

impl<K, O> FixedOutput for UnsupportedHmac<K, O>
where
    K: ArrayLength<u8> + 'static,
    O: ArrayLength<u8> + 'static,
{
    fn finalize_into(self, _: &mut Output<Self>) {
        unreachable!()
    }
}

impl<K, O> Support<bool> for UnsupportedHmac<K, O>
where
    K: ArrayLength<u8> + 'static,
    O: ArrayLength<u8> + 'static,
{
    const SUPPORT: bool = false;
}

#[cfg(feature = "software-crypto-sha256")]
impl crate::Supported for sha2::Sha256 {}

#[cfg(feature = "software-crypto-sha384")]
impl crate::Supported for sha2::Sha384 {}

#[cfg(feature = "internal-hmac")]
impl<D: Default + BlockSizeUser + Update + FixedOutput + HashMarker> crate::Supported
    for hmac::SimpleHmac<D>
{
}
