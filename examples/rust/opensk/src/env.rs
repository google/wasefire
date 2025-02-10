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
// limitations under the License.use opensk_lib::api::clock::Clock;

use opensk_lib::api::crypto::software_crypto::SoftwareCrypto;
use opensk_lib::api::customization::{CustomizationImpl, DEFAULT_CUSTOMIZATION};
use opensk_lib::env::Env;

mod clock;
mod hid_connection;
mod persist;
mod rng;
mod user_presence;
mod write;

#[derive(Default)]
struct WasefireEnv {
    // write: write::WasefireWrite,
    // store: Store<Storage<S, C>>,
    // upgrade_storage: Option<UpgradeStorage<S, C>>,
    // main_connection: WasefireHidConnection,
    // vendor_connection: WasefireHidConnection,
    // blink_pattern: usize,
    // clock: clock::WasefireClock,
    // c: PhantomData<C>,
}

impl Env for WasefireEnv {
    type Rng = Self;
    type UserPresence = Self;
    type Persist = Self;
    type KeyStore = Self;
    type Write = write::Impl;
    type Customization = CustomizationImpl;
    type HidConnection = Self;
    type Clock = Self;
    type Crypto = SoftwareCrypto; // TODO: Use wasefire.

    fn rng(&mut self) -> &mut Self::Rng {
        self
    }

    fn user_presence(&mut self) -> &mut Self::UserPresence {
        self
    }

    fn persist(&mut self) -> &mut Self::Persist {
        self
    }

    fn key_store(&mut self) -> &mut Self::KeyStore {
        self
    }

    fn clock(&mut self) -> &mut Self::Clock {
        self
    }

    fn write(&mut self) -> Self::Write {
        write::Impl::default()
    }

    fn customization(&self) -> &Self::Customization {
        &DEFAULT_CUSTOMIZATION
    }

    fn hid_connection(&mut self) -> &mut Self::HidConnection {
        self
    }

    fn boots_after_soft_reset(&self) -> bool {
        todo!()
    }
}

impl opensk_lib::api::key_store::Helper for WasefireEnv {}
