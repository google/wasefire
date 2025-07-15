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

#[cfg(feature = "applet-api-crypto-cbc")]
mod cbc;
#[cfg(feature = "applet-api-crypto-ccm")]
mod ccm;
#[cfg(feature = "applet-api-crypto-ec")]
mod ec;
#[cfg(feature = "applet-api-crypto-ecdsa")]
mod ecdsa;
#[cfg(feature = "applet-api-crypto-gcm")]
mod gcm;
#[cfg(feature = "internal-applet-api-crypto-hash")]
mod hash;

use wasefire_applet_api::crypto::Api;
use wasefire_board_api::Api as Board;

use crate::DispatchSchedulerCall;

pub fn process<B: Board>(call: Api<DispatchSchedulerCall<B>>) {
    match call {
        #[cfg(feature = "applet-api-crypto-cbc")]
        Api::Cbc(call) => cbc::process(call),
        #[cfg(feature = "applet-api-crypto-ccm")]
        Api::Ccm(call) => ccm::process(call),
        #[cfg(feature = "applet-api-crypto-ec")]
        Api::Ec(call) => ec::process(call),
        #[cfg(feature = "applet-api-crypto-ecdsa")]
        Api::Ecdsa(call) => ecdsa::process(call),
        #[cfg(feature = "applet-api-crypto-gcm")]
        Api::Gcm(call) => gcm::process(call),
        #[cfg(feature = "internal-applet-api-crypto-hash")]
        Api::Hash(call) => hash::process(call),
    }
}
