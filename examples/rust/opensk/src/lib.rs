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

use alloc::vec::Vec;
use core::num::NonZeroU32;

use opensk::api::attestation_store::{Attestation, AttestationStore, Id};
use opensk::api::clock::Clock;
use opensk::api::connection::HidConnection;
use opensk::api::crypto::software_crypto::SoftwareCrypto;
use opensk::api::crypto::Crypto;
// use opensk::api::customization::{CustomizationImpl, AAGUID_LENGTH, DEFAULT_CUSTOMIZATION};
use opensk::api::customization::CustomizationImpl;
use opensk::api::key_store::{CredentialSource, KeyStore};
use opensk::api::rng::Rng;
use opensk::api::user_presence::UserPresence;
use opensk::ctap::secret::Secret;
use opensk::env::Env;
use persistent_store::{Storage, Store};
use rand_core::{impls, CryptoRng, Error, RngCore};
// use wasefire::clock::{Handler, Timer};

fn main() {
    debug!("hello world");
}

// --- RNG

struct WasefireRng;

impl Default for WasefireRng {
    fn default() -> Self {
        Self {}
    }
}

impl CryptoRng for WasefireRng {}

impl RngCore for WasefireRng {
    fn next_u32(&mut self) -> u32 {
        impls::next_u32_via_fill(self)
    }

    fn next_u64(&mut self) -> u64 {
        impls::next_u64_via_fill(self)
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        let _ = rng::fill_bytes(dest);
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), Error> {
        match rng::fill_bytes(dest) {
            Ok(()) => Ok(()),
            Err(_) => Err(NonZeroU32::new(rand_core::Error::CUSTOM_START).unwrap().into()),
        }
    }
}

impl Rng for WasefireRng {}

// --- CUSTOMIZATION

// pub const AAGUID: &[u8; AAGUID_LENGTH] = todo!();
// include_bytes!(concat!(env!("OUT_DIR"), "/opensk_aaguid.bin"));

// const WASEFIRE_CUSTOMIZATION: CustomizationImpl =
//     CustomizationImpl { aaguid: AAGUID, ..DEFAULT_CUSTOMIZATION };

// --- WRITE

pub struct WasefireWrite;

impl Default for WasefireWrite {
    fn default() -> Self {
        Self {}
    }
}

impl core::fmt::Write for WasefireWrite {
    fn write_str(&mut self, _: &str) -> core::fmt::Result {
        todo!()
    }
}

// --- STORE

pub struct WasefireStore;

impl Default for WasefireStore {
    fn default() -> Self {
        Self {}
    }
}

// --- STORAGE

pub struct WasefireStorage;

impl Storage for WasefireStorage {
    fn word_size(&self) -> usize {
        todo!()
    }

    fn page_size(&self) -> usize {
        todo!()
    }

    fn num_pages(&self) -> usize {
        todo!()
    }

    fn max_word_writes(&self) -> usize {
        todo!()
    }

    fn max_page_erases(&self) -> usize {
        todo!()
    }

    fn read_slice(
        &self, index: persistent_store::StorageIndex, length: usize,
    ) -> persistent_store::StorageResult<alloc::borrow::Cow<[u8]>> {
        todo!()
    }

    fn write_slice(
        &mut self, index: persistent_store::StorageIndex, value: &[u8],
    ) -> persistent_store::StorageResult<()> {
        todo!()
    }

    fn erase_page(&mut self, page: usize) -> persistent_store::StorageResult<()> {
        todo!()
    }
}

impl Default for WasefireStorage {
    fn default() -> Self {
        Self {}
    }
}

// --- CLOCK

pub struct WasefireClock {
    // pub struct WasefireClock<H: Handler> {
    // timer: Timer<H>,
}

impl Default for WasefireClock {
    fn default() -> Self {
        // TODO: Replace with actual impl
        // todo!()
        // let timer: Timer<H>
        // let timer = Timer {
        //   // 0,
        //   // Cell::new(false),
        //   // H
        // };
        Self {}
    }
}

impl Clock for WasefireClock {
    type Timer = Self;

    fn make_timer(&mut self, milliseconds: usize) -> Self::Timer {
        todo!()
    }

    fn is_elapsed(&mut self, timer: &Self::Timer) -> bool {
        todo!()
    }
}

// --- USER_PRESENCE

pub struct WasefireUserPresence;

impl Default for WasefireUserPresence {
    fn default() -> Self {
        Self {}
    }
}

impl UserPresence for WasefireUserPresence {
    fn check_init(&mut self) {
        todo!()
    }

    fn wait_with_timeout(
        &mut self, timeout_ms: usize,
    ) -> opensk::api::user_presence::UserPresenceResult {
        todo!()
    }

    fn check_complete(&mut self) {
        todo!()
    }
}

// --- KEYSTORE

pub struct WasefireKeyStore;

impl KeyStore for WasefireKeyStore {
    fn init(&mut self) -> Result<(), opensk::api::key_store::Error> {
        todo!()
    }

    fn wrap_key<E>(
        &mut self,
    ) -> Result<<<E as Env>::Crypto as Crypto>::Aes256, opensk::api::key_store::Error>
    where E: Env {
        todo!()
    }

    fn wrap_credential(
        &mut self, _: CredentialSource,
    ) -> Result<Vec<u8>, opensk::api::key_store::Error> {
        todo!()
    }

    fn unwrap_credential(
        &mut self, _: &[u8], _: &[u8],
    ) -> Result<Option<CredentialSource>, opensk::api::key_store::Error> {
        todo!()
    }

    fn cred_random(&mut self, _: bool) -> Result<Secret<[u8; 32]>, opensk::api::key_store::Error> {
        todo!()
    }

    fn encrypt_pin_hash(
        &mut self, _: &[u8; 16],
    ) -> Result<[u8; 16], opensk::api::key_store::Error> {
        todo!()
    }

    fn decrypt_pin_hash(
        &mut self, _: &[u8; 16],
    ) -> Result<Secret<[u8; 16]>, opensk::api::key_store::Error> {
        todo!()
    }

    fn reset(&mut self) -> Result<(), opensk::api::key_store::Error> {
        todo!()
    }
}

impl Default for WasefireKeyStore {
    fn default() -> Self {
        Self {}
    }
}

// --- HID_CONNECTION

#[derive(Clone, Copy)]
pub struct WasefireHidConnection;

impl HidConnection for WasefireHidConnection {
    fn send_and_maybe_recv(
        &mut self, buf: &mut [u8; 64], timeout_ms: usize,
    ) -> opensk::api::connection::SendOrRecvResult {
        todo!()
    }
}

impl Default for WasefireHidConnection {
    fn default() -> Self {
        Self {}
    }
}

struct WasefireEnv {
    rng: WasefireRng,
    write: WasefireWrite,
    // store: Store<Storage<S, C>>,
    store: WasefireStore,
    storage: WasefireStorage,
    // upgrade_storage: Option<UpgradeStorage<S, C>>,
    main_connection: WasefireHidConnection,
    vendor_connection: WasefireHidConnection,
    // blink_pattern: usize,
    // clock: TockClock<S>,
    clock: WasefireClock,
    // c: PhantomData<C>,
}

impl AttestationStore for WasefireEnv {
    fn get(
        &mut self, id: &Id,
    ) -> Result<Option<Attestation>, opensk::api::attestation_store::Error> {
        todo!()
    }

    fn set(
        &mut self, id: &Id, attestation: Option<&Attestation>,
    ) -> Result<(), opensk::api::attestation_store::Error> {
        todo!()
    }
}

impl Default for WasefireEnv {
    fn default() -> Self {
        let rng = WasefireRng::default();
        let write = WasefireWrite::default();
        let store = WasefireStore::default();
        let storage = WasefireStorage::default();
        let clock = WasefireClock::default();
        let hid_connection = WasefireHidConnection::default();
        WasefireEnv {
            rng,
            write,
            store,
            storage,
            clock,
            main_connection: hid_connection.clone(),
            vendor_connection: hid_connection,
        }
    }
}

impl Env for WasefireEnv {
    // TODO: Some of these are `Self` instead of separate structs in TockEnv &
    // TestEnv. e.g. KeyStore, AttestationStore. Do we need them to be that way?
    type Rng = WasefireRng;
    type Customization = CustomizationImpl;
    type UserPresence = WasefireUserPresence;
    type Storage = WasefireStorage;
    type KeyStore = WasefireKeyStore;
    type Write = WasefireWrite;
    type HidConnection = WasefireHidConnection;
    type AttestationStore = Self;
    type Clock = WasefireClock;
    type Crypto = SoftwareCrypto;

    fn rng(&mut self) -> &mut Self::Rng {
        &mut self.rng
    }

    fn customization(&self) -> &Self::Customization {
        todo!()
        // &WASEFIRE_CUSTOMIZATION
    }

    fn user_presence(&mut self) -> &mut Self::UserPresence {
        todo!()
    }

    fn store(&mut self) -> &mut Store<Self::Storage> {
        todo!()
    }

    fn key_store(&mut self) -> &mut Self::KeyStore {
        todo!()
    }

    fn attestation_store(&mut self) -> &mut Self::AttestationStore {
        todo!()
    }

    fn clock(&mut self) -> &mut Self::Clock {
        todo!()
    }

    fn write(&mut self) -> Self::Write {
        todo!()
    }

    fn main_hid_connection(&mut self) -> &mut Self::HidConnection {
        todo!()
    }
}

#[cfg(test)]
mod test {

    use opensk::api::crypto::ecdsa::SecretKey;
    // use opensk::api::customizaton::is_valid;
    use opensk::api::private_key::PrivateKey;
    use opensk::ctap::data_formats::CoseKey;
    use wasefire_stub as _;

    use super::*;

    // #[test]
    // fn test_invariants() {
    // assert!(is_valid(&WASEFIRE_CUSTOMIZATION));
    // }

    // TODO: Add tests for Rng, Customization and others

    #[test]
    fn test_private_key_get_pub_key() {
        // let mut env = TestEnv::default();
        let mut env: WasefireEnv = WasefireEnv::default();
        let private_key = PrivateKey::new_ecdsa(&mut env);
        let ecdsa_key = private_key.ecdsa_key::<WasefireEnv>().unwrap();
        let public_key = ecdsa_key.public_key();
        assert_eq!(
            private_key.get_pub_key::<WasefireEnv>(),
            Ok(CoseKey::from_ecdsa_public_key(public_key))
        );
    }
}
