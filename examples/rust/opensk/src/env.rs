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

use opensk_lib::env::Env;

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
    // type Rng = Self;
    // type Customization = CustomizationImpl;
    // type UserPresence = Self;
    // type KeyStore = Self;
    // type Persist = Self;
    // type Write = write::WasefireWrite;
    // type HidConnection = WasefireHidConnection;
    // type Clock = clock::WasefireClock;
    // // TODO: We should use wasefire crypto here instead.
    // type Crypto = SoftwareCrypto;

    // fn rng(&mut self) -> &mut Self::Rng {
    //     self
    // }

    // fn customization(&self) -> &Self::Customization {
    //     &WASEFIRE_CUSTOMIZATION
    // }

    // fn user_presence(&mut self) -> &mut Self::UserPresence {
    //     self
    // }

    // fn key_store(&mut self) -> &mut Self::KeyStore {
    //     self
    // }

    // fn clock(&mut self) -> &mut Self::Clock {
    //     &mut self.clock
    // }

    // fn write(&mut self) -> Self::Write {
    //     Self::Write::default()
    // }

    // fn main_hid_connection(&mut self) -> &mut Self::HidConnection {
    //     todo!()
    // }

    // fn persist(&mut self) -> &mut Self::Persist {
    //     self
    // }

    // fn boots_after_soft_reset(&self) -> bool {
    //     false
    // }
}
