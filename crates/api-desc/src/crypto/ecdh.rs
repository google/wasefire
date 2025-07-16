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
        /// ECDH.
    };
    let name = "ecdh".into();
    let items = vec![
        item! {
            enum Curve {
                P256 = 0,
                P384 = 1,
            }
        },
        item! {
            /// The kind of objects for [get_layout].
            enum Kind {
                /// A private key.
                Private = 0,

                /// A public key.
                Public = 1,

                /// A shared secret.
                Shared = 2,
            }
        },
        item! {
            /// Whether a curve is supported.
            fn is_supported "cis" {
                /// The enum value of the [curve][super::Curve].
                curve: usize,
            } -> bool
        },
        item! {
            /// Returns the layout of an object.
            ///
            /// All pointers to such objects must have the appropriate layout.
            fn get_layout "cil" {
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
            /// Generates a private key.
            fn generate "cig" {
                /// The curve.
                curve: usize,

                /// The private key.
                private: *mut u8,
            } -> ()
        },
        item! {
            /// Returns the public key of a private key.
            fn public "cip" {
                /// The curve.
                curve: usize,

                /// The private key.
                private: *const u8,

                /// The public key.
                public: *mut u8,
            } -> ()
        },
        item! {
            /// Computes the shared secret of a private and public key.
            fn shared "cia" {
                /// The curve.
                curve: usize,

                /// The private key.
                private: *const u8,

                /// The public key.
                public: *const u8,

                /// The shared secret.
                shared: *mut u8,
            } -> ()
        },
        item! {
            /// Drops an object.
            fn drop "cid" {
                /// The curve.
                curve: usize,

                /// The kind of object (private or shared).
                kind: usize,

                /// The object.
                object: *mut u8,
            } -> ()
        },
        item! {
            /// Exports a public key.
            fn export "cie" {
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
            fn import "cii" {
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
        item! {
            /// Exports a shared secret.
            fn access "cix" {
                /// The curve.
                curve: usize,

                /// The shared secret.
                shared: *const u8,

                /// The x coordinate in big-endian.
                x: *mut u8,
            } -> ()
        },
    ];
    Item::Mod(Mod { docs, name, items })
}
