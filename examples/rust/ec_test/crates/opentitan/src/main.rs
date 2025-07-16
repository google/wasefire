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

use ecdh_vectors as ecdh;
use elliptic_curve::generic_array::typenum::Unsigned;
use elliptic_curve::sec1::{ModulusSize, ToEncodedPoint};
use elliptic_curve::{
    AffinePoint, CurveArithmetic, FieldBytes, FieldBytesSize, Scalar, ScalarPrimitive, SecretKey,
};

fn main() {
    check_ecdh::<p256::NistP256>("p256", &MASK_P256, ecdh::P256_VECTORS);
    check_ecdh::<p384::NistP384>("p384", &MASK_P384, ecdh::P384_VECTORS);
}

fn check_ecdh<C>(curve: &str, mask: &[u8], vectors: &[ecdh::Vector])
where
    C: CurveArithmetic,
    AffinePoint<C>: ToEncodedPoint<C>,
    FieldBytesSize<C>: ModulusSize,
{
    for &ecdh::Vector { private, otprivate, .. } in vectors {
        check::<C>(curve, "ecdh", mask, private, otprivate);
    }
}

fn check<C>(curve: &str, algo: &str, mask: &[u8], private: &[u8], actual: &[u8])
where
    C: CurveArithmetic,
    AffinePoint<C>: ToEncodedPoint<C>,
    FieldBytesSize<C>: ModulusSize,
{
    let d: Scalar<C> = ScalarPrimitive::<C>::from_slice(private).unwrap().into();
    let d0: Scalar<C> = ScalarPrimitive::<C>::from_slice(mask).unwrap().into();
    let d1 = d - d0;
    let mut d0_bytes: FieldBytes<C> = d0.into();
    let mut d1_bytes: FieldBytes<C> = d1.into();
    d0_bytes.reverse();
    d1_bytes.reverse();
    let privkey = SecretKey::<C>::from_slice(private).unwrap();
    let pubkey = privkey.public_key().as_affine().to_encoded_point(false);
    let mut x_bytes = pubkey.x().unwrap().clone();
    let mut y_bytes = pubkey.y().unwrap().clone();
    x_bytes.reverse();
    y_bytes.reverse();
    let mut expected = Vec::with_capacity(4 * FieldBytesSize::<C>::USIZE + 24);
    expected.extend_from_slice(&d0_bytes);
    expected.extend_from_slice(&[0; 8]); // extra bits
    expected.extend_from_slice(&d1_bytes);
    expected.extend_from_slice(&[0; 8]); // extra bits
    expected.extend_from_slice(&[0; 4]); // checksum
    expected.extend_from_slice(&x_bytes);
    expected.extend_from_slice(&y_bytes);
    expected.extend_from_slice(&[0; 4]); // checksum
    if actual != expected {
        println!("Mismatch for {curve}-{algo}:");
        println!(" private: {private:02x?}");
        println!("  actual: {actual:02x?}");
        println!("expected: {expected:02x?}");
        panic!();
    }
}

// sw/host/tests/crypto/ecdh_kat/src/main.rs
const MASK_P256: [u8; 32] = [
    0x37, 0xc9, 0xe5, 0xb8, 0xe9, 0xe2, 0x44, 0x02, 0xf2, 0xec, 0x25, 0xa2, 0xee, 0xc8, 0x7c, 0x1c,
    0x53, 0x1d, 0x67, 0xe3, 0x8c, 0x18, 0x87, 0x6d, 0x70, 0xaa, 0x50, 0xdd, 0x92, 0x52, 0x65, 0xb1,
];
const MASK_P384: [u8; 48] = [
    0xb8, 0x0f, 0xee, 0x36, 0x25, 0x2d, 0x2c, 0x38, 0x35, 0x0a, 0xd1, 0xc0, 0x08, 0x03, 0xe0, 0x9c,
    0x90, 0xf4, 0xc0, 0x86, 0xe4, 0xcf, 0xee, 0xd7, 0x8f, 0x16, 0x4b, 0x20, 0xb1, 0x00, 0xd5, 0xd4,
    0x5f, 0x16, 0xb6, 0x78, 0xa4, 0x0a, 0x64, 0x29, 0x54, 0x38, 0xbb, 0xeb, 0xc3, 0xe2, 0x9b, 0x09,
];
