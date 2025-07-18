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
        /// Ed25519.
    };
    let name = "ed25519".into();
    let items = vec![
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
            /// Whether Ed25519 is supported.
            fn is_supported "c2s" {} -> bool
        },
        item! {
            /// Returns the layout of an object.
            ///
            /// All pointers to such objects must have the appropriate layout.
            fn get_layout "c2l" {
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
            fn wrapped_length "c2k" {} -> usize
        },
        item! {
            /// Generates a private key.
            fn generate "c2g" {
                /// The private key.
                private: *mut u8,
            } -> ()
        },
        item! {
            /// Returns the public key of a private key.
            fn public "c2p" {
                /// The private key.
                private: *const u8,

                /// The public key.
                public: *mut u8,
            } -> ()
        },
        item! {
            /// Signs a message.
            fn sign "c2i" {
                /// The private key.
                private: *const u8,

                /// The message.
                message: *const u8,

                /// The message length (in bytes).
                message_len: usize,

                /// The first part of the signature.
                r: *mut u8,

                /// The second part of the signature.
                s: *mut u8,
            } -> ()
        },
        item! {
            /// Verifies a signature.
            fn verify "c2v" {
                /// The public key.
                public: *const u8,

                /// The message.
                message: *const u8,

                /// The message length (in bytes).
                message_len: usize,

                /// The first part of the signature.
                r: *const u8,

                /// The second part of the signature.
                s: *const u8,
            } -> bool
        },
        item! {
            /// Drops a private key.
            fn drop "c2d" {
                /// The private key.
                private: *mut u8,
            } -> ()
        },
        item! {
            /// Wraps a private key.
            fn wrap "c2w" {
                /// The private key.
                private: *const u8,

                /// The wrapped private key.
                wrapped: *mut u8,
            } -> ()
        },
        item! {
            /// Unwraps a private key.
            fn unwrap "c2u" {
                /// The wrapped private key.
                wrapped: *const u8,

                /// The private key.
                private: *mut u8,
            } -> ()
        },
        item! {
            /// Exports a public key.
            fn export "c2e" {
                /// The input public key.
                public: *const u8,

                /// The output public key (32 bytes).
                a: *mut u8,
            } -> ()
        },
        item! {
            /// Imports a public key.
            fn import "c2m" {
                /// The input public key (32 bytes).
                a: *const u8,

                /// The output public key.
                public: *mut u8,
            } -> ()
        },
    ];
    Item::Mod(Mod { docs, name, items })
}
