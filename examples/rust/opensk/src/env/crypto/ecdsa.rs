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

use alloc::vec::Vec;

use opensk_lib::api::crypto::ecdsa::{self, Ecdsa};
use opensk_lib::api::crypto::{EC_FIELD_SIZE, EC_SIGNATURE_SIZE, HASH_SIZE};
use opensk_lib::api::rng::Rng;
use wasefire::crypto::ec::{EcdsaPrivate, EcdsaPublic, EcdsaSignature, Int, P256};

use crate::env::WasefireEnv;

impl Ecdsa for WasefireEnv {
    type SecretKey = SecretKey;
    type PublicKey = PublicKey;
    type Signature = Signature;
}

pub struct SecretKey(EcdsaPrivate<P256>);
pub struct PublicKey(EcdsaPublic<P256>);
pub struct Signature(EcdsaSignature<P256>);

impl ecdsa::SecretKey for SecretKey {
    type PublicKey = PublicKey;
    type Signature = Signature;

    fn random(_rng: &mut impl Rng) -> Self {
        // TODO: Use the rng argument.
        SecretKey(EcdsaPrivate::random().unwrap())
    }

    fn from_slice(bytes: &[u8; EC_FIELD_SIZE]) -> Option<Self> {
        Some(SecretKey(EcdsaPrivate::from_non_zero_scalar((*bytes).into()).ok()?))
    }

    fn public_key(&self) -> Self::PublicKey {
        PublicKey(self.0.public_key())
    }

    fn sign(&self, message: &[u8]) -> Self::Signature {
        Signature(self.0.sign(message).unwrap())
    }

    fn to_slice(&self, bytes: &mut [u8; EC_FIELD_SIZE]) {
        *bytes = (*self.0.private_key()).into();
    }
}

impl ecdsa::PublicKey for PublicKey {
    type Signature = Signature;

    fn from_coordinates(x: &[u8; EC_FIELD_SIZE], y: &[u8; EC_FIELD_SIZE]) -> Option<Self> {
        Some(PublicKey(EcdsaPublic::from_coordinates((*x).into(), (*y).into()).ok()?))
    }

    fn verify(&self, message: &[u8], signature: &Self::Signature) -> bool {
        self.0.verify(message, &signature.0).unwrap()
    }

    fn verify_prehash(&self, prehash: &[u8; HASH_SIZE], signature: &Self::Signature) -> bool {
        self.0.verify_prehash(prehash.into(), &signature.0).unwrap()
    }

    fn to_coordinates(&self, x: &mut [u8; EC_FIELD_SIZE], y: &mut [u8; EC_FIELD_SIZE]) {
        *x = (*self.0.x()).into();
        *y = (*self.0.y()).into();
    }
}

impl ecdsa::Signature for Signature {
    fn from_slice(bytes: &[u8; EC_SIGNATURE_SIZE]) -> Option<Self> {
        let r = Int::<P256>::clone_from_slice(&bytes[.. 32]);
        let s = Int::<P256>::clone_from_slice(&bytes[32 ..]);
        Some(Signature(EcdsaSignature::from_components(r, s)))
    }

    fn to_slice(&self, bytes: &mut [u8; EC_SIGNATURE_SIZE]) {
        bytes[.. 32].copy_from_slice(self.0.r());
        bytes[32 ..].copy_from_slice(self.0.s());
    }

    fn to_der(&self) -> Vec<u8> {
        let r = der_int(self.0.r());
        let s = der_int(self.0.s());
        let mut der = Vec::with_capacity(72);
        der.extend([0x30, (2 + r.len() + 2 + s.len()) as u8]); // seq
        der.extend([0x02, r.len() as u8]); // int
        der.extend(r);
        der.extend([0x02, s.len() as u8]); // int
        der.extend(s);
        der
    }
}

fn der_int(x: &Int<P256>) -> Vec<u8> {
    let mut x = x.as_slice().to_vec();
    x.push(0);
    while x.last() == Some(&0) && x.get(x.len() - 2).is_some_and(|x| x & 0x80 != 0) {
        x.pop();
    }
    x.reverse();
    x
}
