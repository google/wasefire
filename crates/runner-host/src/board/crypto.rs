// Copyright 2024 Google LLC
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

use wasefire_board_api::crypto::{self, Api};

use crate::board::Board;

pub enum Impl {}

impl Api for Impl {
    type Aes128Ccm = crypto::SoftwareAes128Ccm;
    type Aes256Cbc = crypto::SoftwareAes256Cbc;
    type Aes256Gcm = crypto::SoftwareAes256Gcm;
    type HmacSha256 = crypto::SoftwareHmacSha256<Self>;
    type HmacSha384 = crypto::SoftwareHmacSha384<Self>;
    type P256 = crypto::SoftwareP256<Self>;
    type P256Ecdsa = crypto::SoftwareP256Ecdsa<Self, CryptoRng>;
    type P384 = crypto::SoftwareP384<Self>;
    type P384Ecdsa = crypto::SoftwareP384Ecdsa<Self, CryptoRng>;
    type Sha256 = crypto::SoftwareSha256;
    type Sha384 = crypto::SoftwareSha384;
}

type CryptoRng = crypto::CryptoRng<Board>;
