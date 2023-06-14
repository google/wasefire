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

use crypto_common::{BlockSizeUser, Key, KeyInit, KeySizeUser, Output, OutputSizeUser};
use digest::{FixedOutput, HashMarker, MacMarker, Update};
use typenum::{U0, U12, U13, U16, U32, U4, U48};

use crate::{Support, Unsupported};

pub mod aead;
pub mod ecc;

/// Cryptography interface.
pub trait Api {
    type Aes128Ccm: aead::Api<U16, U13, U4>;
    type Aes256Gcm: aead::Api<U32, U12, U16>;

    type HmacSha256: Support<bool> + KeyInit + Update + FixedOutput + MacMarker;
    type HmacSha384: Support<bool> + KeyInit + Update + FixedOutput + MacMarker;

    type P256: Support<bool> + ecc::Api<U32>;
    type P384: Support<bool> + ecc::Api<U48>;

    type Sha256: Support<bool> + Default + BlockSizeUser + Update + FixedOutput + HashMarker;
    type Sha384: Support<bool> + Default + BlockSizeUser + Update + FixedOutput + HashMarker;
}

pub type Aes128Ccm<B> = <super::Crypto<B> as Api>::Aes128Ccm;
pub type Aes256Gcm<B> = <super::Crypto<B> as Api>::Aes256Gcm;
pub type HmacSha256<B> = <super::Crypto<B> as Api>::HmacSha256;
pub type HmacSha384<B> = <super::Crypto<B> as Api>::HmacSha384;
pub type P256<B> = <super::Crypto<B> as Api>::P256;
pub type P384<B> = <super::Crypto<B> as Api>::P384;
pub type Sha256<B> = <super::Crypto<B> as Api>::Sha256;
pub type Sha384<B> = <super::Crypto<B> as Api>::Sha384;

macro_rules! software {
    (#[cfg(feature = $feature:literal)] type $Name:ident = $impl:ty;) => {
        #[cfg(feature = $feature)]
        type $Name = $impl;
        #[cfg(not(feature = $feature))]
        type $Name = Unsupported;
    };
}

impl Api for Unsupported {
    software! {
        #[cfg(feature = "software-crypto-aes128-ccm")]
        type Aes128Ccm = ccm::Ccm<aes::Aes128, U4, U13>;
    }
    software! {
        #[cfg(feature = "software-crypto-aes256-gcm")]
        type Aes256Gcm = aes_gcm::Aes256Gcm;
    }

    software! {
        #[cfg(feature = "software-crypto-hmac-sha256")]
        type HmacSha256 = hmac::SimpleHmac<Self::Sha256>;
    }
    software! {
        #[cfg(feature = "software-crypto-hmac-sha384")]
        type HmacSha384 = hmac::SimpleHmac<Self::Sha384>;
    }

    software! {
        #[cfg(feature = "software-crypto-p256")]
        type P256 = ecc::Software<p256::NistP256, Self::Sha256>;
    }
    software! {
        #[cfg(feature = "software-crypto-p384")]
        type P384 = ecc::Software<p384::NistP384, Self::Sha384>;
    }

    software! {
        #[cfg(feature = "software-crypto-sha256")]
        type Sha256 = sha2::Sha256;
    }
    software! {
        #[cfg(feature = "software-crypto-sha384")]
        type Sha384 = sha2::Sha384;
    }
}

impl BlockSizeUser for Unsupported {
    type BlockSize = U0;
}

impl KeySizeUser for Unsupported {
    type KeySize = U0;
}

impl OutputSizeUser for Unsupported {
    type OutputSize = U0;
}

impl HashMarker for Unsupported {}
impl MacMarker for Unsupported {}

impl Default for Unsupported {
    fn default() -> Self {
        unreachable!()
    }
}

impl KeyInit for Unsupported {
    fn new(_: &Key<Self>) -> Self {
        unreachable!()
    }
}

impl Update for Unsupported {
    fn update(&mut self, _: &[u8]) {
        unreachable!()
    }
}

impl FixedOutput for Unsupported {
    fn finalize_into(self, _: &mut Output<Self>) {
        unreachable!()
    }
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
