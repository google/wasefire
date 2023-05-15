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

use crate::{Unimplemented, Unsupported};

pub mod aes128_ccm;
pub mod aes256_gcm;
pub mod hmac_sha256;
pub mod hmac_sha384;
pub mod p256;
pub mod p384;
pub mod sha256;
pub mod sha384;

/// Returns this [`Types`] given a [`crate::Types`].
pub type Get<B> = <super::Get<B> as super::Types>::Crypto;

/// Associated types of [`Api`].
pub trait Types {
    type HmacSha256: hmac_sha256::Types;
    type HmacSha384: hmac_sha384::Types;
    type Sha256: sha256::Types;
    type Sha384: sha384::Types;
}

/// Cryptography interface.
pub trait Api<T: Types> {
    type Aes128Ccm<'a>: aes128_ccm::Api
    where Self: 'a;
    fn aes128_ccm(&mut self) -> Self::Aes128Ccm<'_>;

    type Aes256Gcm<'a>: aes256_gcm::Api
    where Self: 'a;
    fn aes256_gcm(&mut self) -> Self::Aes256Gcm<'_>;

    type HmacSha256<'a>: hmac_sha256::Api<T::HmacSha256>
    where Self: 'a;
    fn hmac_sha256(&mut self) -> Self::HmacSha256<'_>;

    type HmacSha384<'a>: hmac_sha384::Api<T::HmacSha384>
    where Self: 'a;
    fn hmac_sha384(&mut self) -> Self::HmacSha384<'_>;

    type P256<'a>: p256::Api
    where Self: 'a;
    fn p256(&mut self) -> Self::P256<'_>;

    type P384<'a>: p384::Api
    where Self: 'a;
    fn p384(&mut self) -> Self::P384<'_>;

    type Sha256<'a>: sha256::Api<T::Sha256>
    where Self: 'a;
    fn sha256(&mut self) -> Self::Sha256<'_>;

    type Sha384<'a>: sha384::Api<T::Sha384>
    where Self: 'a;
    fn sha384(&mut self) -> Self::Sha384<'_>;
}

impl Types for Unimplemented {
    type HmacSha256 = Unimplemented;
    type HmacSha384 = Unimplemented;
    type Sha256 = Unimplemented;
    type Sha384 = Unimplemented;
}

impl Api<Unimplemented> for Unimplemented {
    type Aes128Ccm<'a> = Unimplemented;
    fn aes128_ccm(&mut self) -> Self::Aes128Ccm<'_> {
        unreachable!()
    }

    type Aes256Gcm<'a> = Unimplemented;
    fn aes256_gcm(&mut self) -> Self::Aes256Gcm<'_> {
        unreachable!()
    }

    type HmacSha256<'a> = Unimplemented;
    fn hmac_sha256(&mut self) -> Self::HmacSha256<'_> {
        unreachable!()
    }

    type HmacSha384<'a> = Unimplemented;
    fn hmac_sha384(&mut self) -> Self::HmacSha384<'_> {
        unreachable!()
    }

    type P256<'a> = Unimplemented;
    fn p256(&mut self) -> Self::P256<'_> {
        unreachable!()
    }

    type P384<'a> = Unimplemented;
    fn p384(&mut self) -> Self::P384<'_> {
        unreachable!()
    }

    type Sha256<'a> = Unimplemented;
    fn sha256(&mut self) -> Self::Sha256<'_> {
        unreachable!()
    }

    type Sha384<'a> = Unimplemented;
    fn sha384(&mut self) -> Self::Sha384<'_> {
        unreachable!()
    }
}

impl Types for Unsupported {
    type HmacSha256 = Unsupported;
    type HmacSha384 = Unsupported;
    type Sha256 = Unsupported;
    type Sha384 = Unsupported;
}

impl Api<Unsupported> for Unsupported {
    type Aes128Ccm<'a> = Unsupported;
    fn aes128_ccm(&mut self) -> Self::Aes128Ccm<'_> {
        Unsupported
    }

    type Aes256Gcm<'a> = Unsupported;
    fn aes256_gcm(&mut self) -> Self::Aes256Gcm<'_> {
        Unsupported
    }

    type HmacSha256<'a> = Unsupported;
    fn hmac_sha256(&mut self) -> Self::HmacSha256<'_> {
        Unsupported
    }

    type HmacSha384<'a> = Unsupported;
    fn hmac_sha384(&mut self) -> Self::HmacSha384<'_> {
        Unsupported
    }

    type P256<'a> = Unsupported;
    fn p256(&mut self) -> Self::P256<'_> {
        Unsupported
    }

    type P384<'a> = Unsupported;
    fn p384(&mut self) -> Self::P384<'_> {
        Unsupported
    }

    type Sha256<'a> = Unsupported;
    fn sha256(&mut self) -> Self::Sha256<'_> {
        Unsupported
    }

    type Sha384<'a> = Unsupported;
    fn sha384(&mut self) -> Self::Sha384<'_> {
        Unsupported
    }
}
