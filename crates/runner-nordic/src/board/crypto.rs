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

use wasefire_board_api::crypto;

#[cfg(feature = "aes128-ccm")]
mod ccm;

pub enum Impl {}

impl crypto::Api for Impl {
    #[cfg(feature = "aes128-ccm")]
    type Aes128Ccm = ccm::Impl;
    #[cfg(feature = "software-crypto-aes256-cbc")]
    type Aes256Cbc = crypto::SoftwareAes256Cbc;
    #[cfg(feature = "software-crypto-aes256-gcm")]
    type Aes256Gcm = crypto::SoftwareAes256Gcm;
    #[cfg(feature = "software-crypto-hmac-sha256")]
    type HmacSha256 = crypto::SoftwareHmacSha256<Self>;
    #[cfg(feature = "software-crypto-p256")]
    type P256 = crypto::SoftwareP256<Self>;
    #[cfg(feature = "software-crypto-p256-ecdh")]
    type P256Ecdh = crypto::SoftwareP256Ecdh<crypto::CryptoRng<crate::Board>>;
    #[cfg(feature = "software-crypto-p256-ecdsa")]
    type P256Ecdsa = crypto::SoftwareP256Ecdsa<Self, crypto::CryptoRng<crate::Board>>;
    #[cfg(feature = "software-crypto-sha256")]
    type Sha256 = crypto::SoftwareSha256;
}
