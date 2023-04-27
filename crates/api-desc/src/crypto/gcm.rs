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
        /// AES-256-GCM.
    };
    let name = "gcm".into();
    let items = vec![
        item! {
            /// Whether AES-256-GCM is supported.
            fn is_supported "cgs" {
            } -> {
                /// 1 if supported, 0 otherwise.
                supported: usize,
            }
        },
        item! {
            /// Encrypts and authenticates a clear text with associated data given a key and IV.
            fn encrypt "cge" {
                /// The 32 bytes key.
                key: *const u8,

                /// The 12 bytes IV.
                iv: *const u8,

                /// The additional authenticated data.
                aad: *const u8,

                /// The length of the additional authenticated data.
                aad_len: usize,

                /// The length of the clear (and cipher) text.
                length: usize,

                /// The clear text.
                clear: *const u8,

                /// The cipher text.
                cipher: *mut u8,

                /// The 16 bytes authentication tag.
                tag: *mut u8,
            } -> {
                /// Zero on success, bitwise complement of [`Error`](crate::crypto::Error)
                /// otherwise.
                res: isize,
            }
        },
        item! {
            /// Decrypts and authenticates a cipher text with associated data given a key and IV.
            fn decrypt "cgd" {
                /// The 32 bytes key.
                key: *const u8,

                /// The 12 bytes IV.
                iv: *const u8,

                /// The additional authenticated data.
                aad: *const u8,

                /// The length of the additional authenticated data.
                aad_len: usize,

                /// The 16 bytes authentication tag.
                tag: *const u8,

                /// The length of the cipher (and clear) text.
                length: usize,

                /// The cipher text.
                cipher: *const u8,

                /// The clear text.
                clear: *mut u8,
            } -> {
                /// Zero on success, bitwise complement of [`Error`](crate::crypto::Error)
                /// otherwise.
                res: isize,
            }
        },
    ];
    Item::Mod(Mod { docs, name, items })
}
