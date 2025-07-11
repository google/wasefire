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

use crate::*;

pub(crate) fn new() -> Item {
    let docs = docs! {
        /// ECDSA.
    };
    let name = "ecdsa".into();
    let items = vec![
        item! {
            enum Curve {
                P256 = 0,
                P384 = 1,
            }
        },
        item! {
            /// The kind of objects for import/export.
            enum Kind {
                /// A private key.
                Private = 0,

                /// A public key.
                Public = 1,
            }
        },
        item! {
            /// Whether a curve is supported.
            fn is_supported "cds" {
                /// The enum value of the [curve][super::Curve].
                curve: usize,
            } -> bool
        },
        item! {
            /// Returns the layout of an object.
            ///
            /// All pointers to such objects must have the appropriate layout.
            fn get_layout "cdl" {
                /// The curve.
                curve: usize,

                /// The kind of object.
                kind: usize,

                /// The size.
                size: *mut u32,

                /// The alignment.
                align: *mut u32,
            } -> ()
        },
        item! {
            /// Returns the length of a wrapped private key.
            fn wrapped_length "cdk" {
                /// The curve.
                curve: usize,
            } -> usize
        },
        item! {
            /// Generates a private key.
            fn generate "cdg" {
                /// The curve.
                curve: usize,

                /// The private key.
                private: *mut u8,
            } -> ()
        },
        item! {
            /// Returns the public key of a private key.
            fn public "cdp" {
                /// The curve.
                curve: usize,

                /// The private key.
                private: *const u8,

                /// The public key.
                public: *mut u8,
            } -> ()
        },
        item! {
            /// Signs a pre-hashed message.
            fn sign "cdi" {
                /// The curve.
                curve: usize,

                /// The private key.
                private: *const u8,

                /// The pre-hashed message.
                digest: *const u8,

                /// The first part of the signature.
                r: *mut u8,

                /// The second part of the signature.
                s: *mut u8,
            } -> ()
        },
        item! {
            /// Verifies a signature.
            fn verify "cdv" {
                /// The curve.
                curve: usize,

                /// The public key.
                public: *const u8,

                /// The pre-hashed message.
                digest: *const u8,

                /// The first part of the signature.
                r: *const u8,

                /// The second part of the signature.
                s: *const u8,
            } -> bool
        },
        item! {
            /// Drops a private key.
            fn drop "cdd" {
                /// The curve.
                curve: usize,

                /// The private key.
                private: *mut u8,
            } -> ()
        },
        item! {
            /// Wraps a private key.
            fn wrap "cdw" {
                /// The curve.
                curve: usize,

                /// The private key.
                private: *const u8,

                /// The wrapped private key.
                wrapped: *mut u8,
            } -> ()
        },
        item! {
            /// Unwraps a private key.
            fn unwrap "cdu" {
                /// The curve.
                curve: usize,

                /// The wrapped private key.
                wrapped: *const u8,

                /// The private key.
                private: *mut u8,
            } -> ()
        },
        item! {
            /// Exports a public key.
            fn export "cde" {
                /// The curve.
                curve: usize,

                /// The public key.
                public: *const u8,

                /// The x coordinate in big-endian.
                x: *mut u8,

                /// The y coordinate in big-endian.
                y: *mut u8,
            } -> ()
        },
        item! {
            /// Imports a public key.
            fn import "cdm" {
                /// The curve.
                curve: usize,

                /// The x coordinate in big-endian.
                x: *const u8,

                /// The y coordinate in big-endian.
                y: *const u8,

                /// The public key.
                public: *mut u8,
            } -> ()
        },
    ];
    Item::Mod(Mod { docs, name, items })
}
