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

use crate::*;

pub(crate) fn new() -> Item {
    let docs = docs! {
        /// Elliptic curves.
    };
    let name = "ec".into();
    let items = vec![
        item! {
            enum Curve {
                P256 = 0,
                P384 = 1,
            }
        },
        item! {
            /// Whether a curve is supported.
            fn is_supported "ces" {
                /// The enum value of the [curve][super::Curve].
                curve: usize,
            } -> {
                /// 1 when supported, 0 otherwise.
                support: usize,
            }
        },
        item! {
            /// Returns whether a scalar is valid.
            ///
            /// A scalar is valid if smaller than the field's modulus.
            fn is_valid_scalar "cet" {
                /// The curve.
                curve: usize,

                /// The scalar in SEC1 encoding.
                n: *const u8,
            } -> {
                /// 1 if valid, 0 otherwise.
                valid: usize,
            }
        },
        item! {
            /// Returns whether a point is valid.
            fn is_valid_point "ceq" {
                /// The curve.
                curve: usize,

                /// The x-coordinate in SEC1 encoding.
                x: *const u8,

                /// The y-coordinate in SEC1 encoding.
                y: *const u8,
            } -> {
                /// 1 if valid, 0 otherwise.
                valid: usize,
            }
        },
        item! {
            /// Performs base point multiplication.
            fn base_point_mul "ceb" {
                /// The curve.
                curve: usize,

                /// The scalar in SEC1 encoding.
                n: *const u8,

                /// The x coordinate in SEC1 encoding.
                x: *mut u8,

                /// The y coordinate in SEC1 encoding.
                y: *mut u8,
            } -> {
                /// Zero on success. Negative on error.
                res: isize,
            }
        },
        item! {
            /// Performs point multiplication.
            fn point_mul "cep" {
                /// The curve.
                curve: usize,

                /// The scalar in SEC1 encoding.
                n: *const u8,

                /// The x coordinate of the input point in SEC1 encoding.
                in_x: *const u8,

                /// The y coordinate of the input point in SEC1 encoding.
                in_y: *const u8,

                /// The x coordinate of the output point in SEC1 encoding.
                out_x: *mut u8,

                /// The y coordinate of the output point in SEC1 encoding.
                out_y: *mut u8,
            } -> {
                /// Zero on success. Negative on error.
                res: isize,
            }
        },
        item! {
            /// Signs a message with ECDSA.
            fn ecdsa_sign "cei" {
                /// The curve.
                curve: usize,

                /// The private key scalar in SEC1 encoding.
                key: *const u8,

                /// The integer message in SEC1 encoding.
                message: *const u8,

                /// The r signature component in SEC1 encoding.
                r: *mut u8,

                /// The s signature component in SEC1 encoding.
                s: *mut u8,
            } -> {
                /// Zero on success. Negative on error.
                res: isize,
            }
        },
        item! {
            /// Verifies an ECDSA signature.
            fn ecdsa_verify "cev" {
                /// The curve.
                curve: usize,

                /// The integer message in SEC1 encoding.
                message: *const u8,

                /// The x-coordinate in SEC1 encoding.
                x: *const u8,

                /// The y-coordinate in SEC1 encoding.
                y: *const u8,

                /// The r signature component in SEC1 encoding.
                r: *const u8,

                /// The s signature component in SEC1 encoding.
                s: *const u8,
            } -> {
                /// One if the signature is valid. Zero if invalid. Negative on error.
                res: isize,
            }
        },
    ];
    Item::Mod(Mod { docs, name, items })
}
