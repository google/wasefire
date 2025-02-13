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

use opensk_lib::api::crypto::EC_FIELD_SIZE;
use opensk_lib::api::crypto::ecdh::{self, Ecdh};
use opensk_lib::api::rng::Rng;
use wasefire::crypto::ec::{EcdhPrivate, EcdhPublic, EcdhShared, P256};

use crate::env::WasefireEnv;

impl Ecdh for WasefireEnv {
    type SecretKey = SecretKey;
    type PublicKey = PublicKey;
    type SharedSecret = SharedSecret;
}

pub struct SecretKey(EcdhPrivate<P256>);
pub struct PublicKey(EcdhPublic<P256>);
pub struct SharedSecret(EcdhShared<P256>);

impl ecdh::SecretKey for SecretKey {
    type PublicKey = PublicKey;
    type SharedSecret = SharedSecret;

    fn random(_rng: &mut impl Rng) -> Self {
        // TODO: Use the rng argument.
        SecretKey(EcdhPrivate::random().unwrap())
    }

    fn public_key(&self) -> Self::PublicKey {
        PublicKey(self.0.public_key())
    }

    fn diffie_hellman(&self, public: &Self::PublicKey) -> Self::SharedSecret {
        SharedSecret(self.0.diffie_hellman(&public.0))
    }
}

impl ecdh::PublicKey for PublicKey {
    fn from_coordinates(x: &[u8; EC_FIELD_SIZE], y: &[u8; EC_FIELD_SIZE]) -> Option<Self> {
        Some(PublicKey(EcdhPublic::from_coordinates((*x).into(), (*y).into()).ok()?))
    }

    fn to_coordinates(&self, x: &mut [u8; EC_FIELD_SIZE], y: &mut [u8; EC_FIELD_SIZE]) {
        *x = (*self.0.x()).into();
        *y = (*self.0.y()).into();
    }
}

impl ecdh::SharedSecret for SharedSecret {
    fn raw_secret_bytes(&self, secret: &mut [u8; EC_FIELD_SIZE]) {
        *secret = (*self.0.raw_bytes()).into();
    }
}
