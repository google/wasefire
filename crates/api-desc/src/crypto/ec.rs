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
                /// Zero on success, bitwise complement of [`Error`](crate::crypto::Error)
                /// otherwise.
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
                out_x: *const u8,

                /// The y coordinate of the output point in SEC1 encoding.
                out_y: *const u8,
            } -> {
                /// Zero on success, bitwise complement of [`Error`](crate::crypto::Error)
                /// otherwise.
                res: isize,
            }
        },
    ];
    Item::Mod(Mod { docs, name, items })
}
