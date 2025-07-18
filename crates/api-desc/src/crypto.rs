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

use crate::*;

#[cfg(feature = "api-crypto-cbc")]
mod cbc;
#[cfg(feature = "api-crypto-ccm")]
mod ccm;
#[cfg(feature = "api-crypto-ec")]
mod ec;
#[cfg(feature = "api-crypto-ecdh")]
mod ecdh;
#[cfg(feature = "api-crypto-ecdsa")]
mod ecdsa;
#[cfg(feature = "api-crypto-ed25519")]
mod ed25519;
#[cfg(feature = "api-crypto-gcm")]
mod gcm;
#[cfg(feature = "internal-api-crypto-hash")]
mod hash;

pub(crate) fn new() -> Item {
    let docs = docs! {
        /// Cryptographic operations.
    };
    let name = "crypto".into();
    let items = vec![
        #[cfg(feature = "api-crypto-cbc")]
        cbc::new(),
        #[cfg(feature = "api-crypto-ccm")]
        ccm::new(),
        #[cfg(feature = "api-crypto-ec")]
        ec::new(),
        #[cfg(feature = "api-crypto-ecdh")]
        ecdh::new(),
        #[cfg(feature = "api-crypto-ecdsa")]
        ecdsa::new(),
        #[cfg(feature = "api-crypto-ed25519")]
        ed25519::new(),
        #[cfg(feature = "api-crypto-gcm")]
        gcm::new(),
        #[cfg(feature = "internal-api-crypto-hash")]
        hash::new(),
    ];
    Item::Mod(Mod { docs, name, items })
}
