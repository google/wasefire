// Copyright 2025 Google LLC
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

//! Tests that ECDH is working properly.

#![no_std]
wasefire::applet!();

use alloc::borrow::Cow;
use alloc::vec;

use ecdh_vectors::{P256_VECTORS, P384_VECTORS, Vector};
use wasefire::crypto::ecdh::{Curve, P256, P384, Private, Public, Shared};
use wasefire_one_of::at_most_one_of;

at_most_one_of!["runner-opentitan", "runner-"];

pub fn main() -> ! {
    test_random::<P256>("p256");
    test_random::<P384>("p384");
    test::<P256>("p256", P256_VECTORS);
    test::<P384>("p384", P384_VECTORS);
    scheduling::exit();
}

fn test_random<C: Curve>(name: &str) {
    debug!("test_{name}_random(): Computes ECDH with random private keys.");
    if !C::is_supported() {
        debug!("- not supported");
        return;
    }
    for _ in 0 .. 5 {
        let k1 = Private::<C>::generate().unwrap();
        let k2 = Private::<C>::generate().unwrap();
        let s1 = Shared::<C>::new(&k1, &k2.public().unwrap()).unwrap();
        let s2 = Shared::<C>::new(&k2, &k1.public().unwrap()).unwrap();
        let mut x1 = vec![0; C::SIZE];
        let mut x2 = vec![0; C::SIZE];
        s1.export(&mut x1).unwrap();
        s2.export(&mut x2).unwrap();
        assert_eq!(x1, x2);
        debug!("- {:02x?}", &x1[.. 8]);
    }
}

fn test<C: Curve>(name: &str, vectors: &[Vector]) {
    debug!("test_{name}(): Computes ECDH on the test vectors.");
    if !C::is_supported() {
        debug!("- not supported");
        return;
    }
    for &Vector { tc_id, private, otprivate, public_x, public_y, shared } in vectors {
        debug!("- {tc_id}");
        let private: Cow<[u8]> = match () {
            () if cfg!(feature = "runner-opentitan") => {
                let mut private = otprivate.to_vec();
                unsafe { vendor::syscall(0x80000001, 1, private.as_mut_ptr().addr(), 0) }.unwrap();
                private.into()
            }
            _ => private.into(),
        };
        let private = Private::<C>::import_testonly(&private).unwrap();
        let public = Public::<C>::import(public_x, public_y).unwrap();
        let shared_ = Shared::<C>::new(&private, &public).unwrap();
        let mut actual = vec![0; C::SIZE];
        shared_.export(&mut actual).unwrap();
        assert_eq!(actual, shared);
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
        test::<P256>("p256", P256_VECTORS);
    }

    #[test]
    fn test_p384() {
        test::<P384>("p384", P384_VECTORS);
    }
}
