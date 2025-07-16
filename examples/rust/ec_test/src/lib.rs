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

//! Tests that elliptic curves are working properly.

#![no_std]
wasefire::applet!();

use alloc::vec::Vec;

use wasefire::crypto::ec::{
    Curve, EcdhPrivate, EcdhPublic, EcdsaPrivate, EcdsaPublic, EcdsaSignature, Int, P256, P384,
};
use {ecdh_vectors as ecdh, ecdsa_vectors as ecdsa};

pub fn main() -> ! {
    test_ecdh::<P256>("p256", ecdh::P256_VECTORS);
    test_ecdh::<P384>("p384", ecdh::P384_VECTORS);
    test_ecdh_random::<P256>("p256");
    test_ecdh_random::<P384>("p384");
    test_ecdsa::<P256>("p256", ecdsa::P256_VECTORS);
    test_ecdsa::<P384>("p384", ecdsa::P384_VECTORS);
    test_ecdsa_random::<P256>("p256");
    test_ecdsa_random::<P384>("p384");
    scheduling::exit();
}

fn test_ecdh<C: Curve>(name: &str, vectors: &[ecdh::Vector]) {
    debug!("test_ecdh_{name}(): Computes ECDH on the test vectors.");
    if !C::is_supported() {
        debug!("- not supported");
        return;
    }
    for &ecdh::Vector { tc_id, private, public_x, public_y, shared, .. } in vectors {
        debug!("- {tc_id}");
        let private =
            EcdhPrivate::<C>::from_non_zero_scalar(Int::<C>::clone_from_slice(private)).unwrap();
        let public = EcdhPublic::<C>::from_coordinates(
            Int::<C>::clone_from_slice(public_x),
            Int::<C>::clone_from_slice(public_y),
        )
        .unwrap();
        let shared_ = private.diffie_hellman(&public);
        assert_eq!(shared_.raw_bytes().as_slice(), shared);
    }
}

fn test_ecdh_random<C: Curve>(name: &str) {
    debug!("test_ecdh_{name}_random(): Computes ECDH with random private keys.");
    if !C::is_supported() {
        debug!("- not supported");
        return;
    }
    for _ in 0 .. 5 {
        let k1 = EcdhPrivate::<C>::random().unwrap();
        let k2 = EcdhPrivate::<C>::random().unwrap();
        let s1 = k1.diffie_hellman(&k2.public_key());
        let s2 = k2.diffie_hellman(&k1.public_key());
        assert_eq!(s1.raw_bytes(), s2.raw_bytes());
        debug!("- {:02x?}", &s1.raw_bytes()[.. 8]);
    }
}

fn test_ecdsa<C: Curve>(name: &str, vectors: &[ecdsa::Vector]) {
    debug!("test_ecdsa_{name}(): Verifies ECDSA signatures of test vectors.");
    if !C::is_supported() {
        debug!("- not supported");
        return;
    }
    for &ecdsa::Vector { x, y, m, r, s, v } in vectors {
        debug!("- {:02x?}", &m[.. 8]);
        let public = EcdsaPublic::<C>::from_coordinates(
            Int::<C>::clone_from_slice(x),
            Int::<C>::clone_from_slice(y),
        )
        .unwrap();
        let signature = EcdsaSignature::from_components(
            Int::<C>::clone_from_slice(r),
            Int::<C>::clone_from_slice(s),
        );
        let v_ = public.verify(m, &signature).unwrap();
        assert_eq!(v_, v);
    }
}

fn test_ecdsa_random<C: Curve>(name: &str) {
    debug!("test_ecdsa_{name}_random(): Computes ECDSA with random private keys.");
    if !C::is_supported() {
        debug!("- not supported");
        return;
    }
    let mut rs = Vec::new();
    for _ in 0 .. 5 {
        let d = EcdsaPrivate::<C>::random().unwrap();
        let mut m = rng::bytes(257).unwrap().into_vec();
        m.truncate(m[256] as usize);
        let s = d.sign(&m).unwrap();
        debug!("- {:02x?}{:02x?} ({} bytes message)", &s.r()[.. 4], &s.s()[.. 4], m.len());
        rs.push(s.r().clone());
        let p = d.public_key();
        assert!(p.verify(&m, &s).unwrap());
    }
    // Make sure all r components are different.
    for i in 1 .. rs.len() {
        for j in 0 .. i {
            assert!(rs[j] != rs[i]);
        }
    }
}

// TODO(https://github.com/rust-lang/rust/issues/95513): Remove when fixed.
#[cfg(feature = "test")]
use wasefire_stub as _;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ecdh_p256() {
        test_ecdh::<P256>("p256", ecdh::P256_VECTORS);
    }

    #[test]
    fn test_ecdh_p384() {
        test_ecdh::<P384>("p384", ecdh::P384_VECTORS);
    }

    #[test]
    fn test_ecdh_random_p256() {
        test_ecdh_random::<P256>("p256");
    }

    #[test]
    fn test_ecdh_random_p384() {
        test_ecdh_random::<P384>("p384");
    }

    #[test]
    fn test_ecdsa_p256() {
        test_ecdsa::<P256>("p256", ecdsa::P256_VECTORS);
    }

    #[test]
    fn test_ecdsa_p384() {
        test_ecdsa::<P384>("p384", ecdsa::P384_VECTORS);
    }

    #[test]
    fn test_ecdsa_random_p256() {
        test_ecdsa_random::<P256>("p256");
    }

    #[test]
    fn test_ecdsa_random_p384() {
        test_ecdsa_random::<P384>("p384");
    }
}
