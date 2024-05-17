// Copyright 2023 Google LLC
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

//! TODO: Add docs

#![no_std]
wasefire::applet!();

use alloc::boxed::Box;
use alloc::vec::Vec;

use opensk_lib::api::connection::{HidConnection, SendOrRecvResult};
use opensk_lib::api::crypto::software_crypto::SoftwareCrypto;
use opensk_lib::api::customization::{CustomizationImpl, AAGUID_LENGTH, DEFAULT_CUSTOMIZATION};
use opensk_lib::api::persist::{Attestation, Persist, PersistIter};
use opensk_lib::ctap::status_code::Ctap2StatusCode::{
    self, CTAP1_ERR_OTHER, CTAP2_ERR_KEY_STORE_FULL, CTAP2_ERR_VENDOR_HARDWARE_FAILURE,
    CTAP2_ERR_VENDOR_INTERNAL_ERROR,
};
use opensk_lib::ctap::status_code::CtapResult;
use opensk_lib::env::Env;
use wasefire_error::{Code, Space};
// use wasefire::clock::{Handler, Timer};

mod clock;
mod keystore;
mod rng;
mod user_presence;
mod write;

fn main() {
    debug!("hello world");
}

pub const AAGUID: &[u8; AAGUID_LENGTH] =
    include_bytes!(concat!(env!("OUT_DIR"), "/opensk_aaguid.bin"));

const WASEFIRE_CUSTOMIZATION: CustomizationImpl =
    CustomizationImpl { aaguid: AAGUID, ..DEFAULT_CUSTOMIZATION };

#[derive(Clone, Copy, Default)]
pub struct WasefireHidConnection;

impl HidConnection for WasefireHidConnection {
    fn send_and_maybe_recv(&mut self, buf: &mut [u8; 64], timeout_ms: usize) -> SendOrRecvResult {
        todo!()
    }
}

#[derive(Default)]
struct WasefireEnv {
    write: write::WasefireWrite,
    // store: Store<Storage<S, C>>,
    // upgrade_storage: Option<UpgradeStorage<S, C>>,
    main_connection: WasefireHidConnection,
    vendor_connection: WasefireHidConnection,
    // blink_pattern: usize,
    clock: clock::WasefireClock,
    // c: PhantomData<C>,
}

fn convert(error: wasefire::Error) -> Ctap2StatusCode {
    match (Space::try_from(error.space()), Code::try_from(error.code())) {
        (_, Ok(Code::NotEnough)) => CTAP2_ERR_KEY_STORE_FULL,
        (Ok(Space::User | Space::Internal), _) => CTAP2_ERR_VENDOR_INTERNAL_ERROR,
        (Ok(Space::World), _) => CTAP2_ERR_VENDOR_HARDWARE_FAILURE,
        _ => CTAP1_ERR_OTHER,
    }
}

impl Persist for WasefireEnv {
    fn find(&self, key: usize) -> CtapResult<Option<Vec<u8>>> {
        match store::find(key) {
            Err(e) => Err(convert(e)),
            Ok(None) => Ok(None),
            Ok(Some(data)) => Ok(Some(data.into_vec())),
        }
    }

    fn insert(&mut self, key: usize, value: &[u8]) -> CtapResult<()> {
        store::insert(key, value).map_err(convert)
    }

    fn remove(&mut self, key: usize) -> CtapResult<()> {
        store::remove(key).map_err(convert)
    }

    fn iter(&self) -> CtapResult<PersistIter<'_>> {
        let keys = store::keys().map_err(convert)?;
        Ok(Box::new(keys.into_iter().map(|x| Ok(x as usize))))
    }
}

impl Env for WasefireEnv {
    type Rng = Self;
    type Customization = CustomizationImpl;
    type UserPresence = Self;
    type KeyStore = Self;
    type Persist = Self;
    type Write = write::WasefireWrite;
    type HidConnection = WasefireHidConnection;
    type Clock = clock::WasefireClock;
    // TODO: We should use wasefire crypto here instead.
    type Crypto = SoftwareCrypto;

    fn rng(&mut self) -> &mut Self::Rng {
        self
    }

    fn customization(&self) -> &Self::Customization {
        &WASEFIRE_CUSTOMIZATION
    }

    fn user_presence(&mut self) -> &mut Self::UserPresence {
        self
    }

    fn key_store(&mut self) -> &mut Self::KeyStore {
        self
    }

    fn clock(&mut self) -> &mut Self::Clock {
        &mut self.clock
    }

    fn write(&mut self) -> Self::Write {
        Self::Write::default()
    }

    fn main_hid_connection(&mut self) -> &mut Self::HidConnection {
        todo!()
    }

    fn persist(&mut self) -> &mut Self::Persist {
        self
    }

    fn boots_after_soft_reset(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {

    use opensk_lib::api::crypto::ecdsa::SecretKey;
    use opensk_lib::api::customization::is_valid;
    use opensk_lib::api::private_key::PrivateKey;
    use opensk_lib::ctap::data_formats::CoseKey;
    use wasefire_stub as _;

    use super::*;

    #[test]
    fn test_invariants() {
        assert!(is_valid(&WASEFIRE_CUSTOMIZATION));
    }

    // TODO: Add tests for Rng, Customization and others

    #[test]
    fn test_private_key_get_pub_key() {
        // let mut env = TestEnv::default();
        let mut env = WasefireEnv::default();
        let private_key = PrivateKey::new_ecdsa(&mut env);
        let ecdsa_key = private_key.ecdsa_key::<WasefireEnv>().unwrap();
        let public_key = ecdsa_key.public_key();
        assert_eq!(
            private_key.get_pub_key::<WasefireEnv>(),
            Ok(CoseKey::from_ecdsa_public_key(public_key))
        );
    }
}
