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

//! Tests that ECDSA is working properly.

#![no_std]
wasefire::applet!();

use alloc::vec;
use alloc::vec::Vec;

use ecdsa_vectors::{EcdsaVector, P256_ECDSA_VECTORS, P384_ECDSA_VECTORS};
use wasefire::crypto::ecdsa::{Curve, P256, P384, Private, Public};

pub fn main() -> ! {
    test_random::<P256>("p256");
    test_random::<P384>("p384");
    test::<P256>("p256", P256_ECDSA_VECTORS);
    test::<P384>("p384", P384_ECDSA_VECTORS);
    scheduling::exit();
}

fn test_random<C: Curve>(name: &str) {
    debug!("test_{name}_random(): Computes ECDSA with random private keys.");
    if !C::is_supported() {
        debug!("- not supported");
        return;
    }
    let mut rs = Vec::new();
    for _ in 0 .. 5 {
        let key = Private::<C>::generate().unwrap();
        let pubkey = key.public();
        let key = Private::<C>::import(&key.export().unwrap()).unwrap();
        let mut msg = rng::bytes(257).unwrap().into_vec();
        msg.truncate(msg[256] as usize);
        let mut r = vec![0; C::SIZE];
        let mut s = vec![0; C::SIZE];
        key.sign(&msg, &mut r, &mut s).unwrap();
        debug!("- {:02x?}{:02x?} ({} bytes message)", &r[.. 4], &s[.. 4], msg.len());
        assert!(pubkey.unwrap().verify(&msg, &r, &s).unwrap());
        rs.push(r);
    }
    // Make sure all r components are different.
    for i in 1 .. rs.len() {
        for j in 0 .. i {
            assert!(rs[j] != rs[i]);
        }
    }
}

fn test<C: Curve>(name: &str, vectors: &[EcdsaVector]) {
    debug!("test_{name}(): Verifies ECDSA signatures of test vectors.");
    if !C::is_supported() {
        debug!("- not supported");
        return;
    }
    for &EcdsaVector { x, y, m, r, s, v } in vectors {
        debug!("- {:02x?}", &m[.. 8]);
        let key = Public::<C>::import(x, y).unwrap();
        assert_eq!(key.verify(m, r, s).unwrap(), v);
        let mut pub_x = vec![0; C::SIZE];
        let mut pub_y = vec![0; C::SIZE];
        key.export(&mut pub_x, &mut pub_y).unwrap();
        assert_eq!(pub_x, x);
        assert_eq!(pub_y, y);
    }
}

// TODO(https://github.com/rust-lang/rust/issues/95513): Remove when fixed.
#[cfg(feature = "test")]
use wasefire_stub as _;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_random_p256() {
        test_random::<P256>("p256");
    }

    #[test]
    fn test_random_p384() {
        test_random::<P384>("p384");
    }

    #[test]
    fn test_p256() {
        test::<P256>("p256", P256_ECDSA_VECTORS);
    }

    #[test]
    fn test_p384() {
        test::<P384>("p384", P384_ECDSA_VECTORS);
    }
}
