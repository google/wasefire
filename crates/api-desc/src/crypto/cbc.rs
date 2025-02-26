// Copyright 2024 Google LLC
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
        /// AES-256-CBC.
    };
    let name = "cbc".into();
    let items = vec![
        item! {
            /// Whether AES-256-CBC is supported.
            fn is_supported "cbs" {} -> bool
        },
        item! {
            /// Encrypts a sequence of blocks given a key and IV.
            fn encrypt "cbe" {
                /// The 32 bytes key to encrypt with.
                key: *const u8,

                /// The 16 bytes IV to encrypt with.
                iv: *const u8,

                /// Address of the sequence of blocks.
                ptr: *mut u8,

                /// Length in bytes of the sequence of blocks.
                ///
                /// This length must be dividable by 16.
                len: usize,
            } -> ()
        },
        item! {
            /// Decrypts a sequence of blocks given a key and IV.
            fn decrypt "cbd" {
                /// The 32 bytes key to decrypt with.
                key: *const u8,

                /// The 16 bytes IV to decrypt with.
                iv: *const u8,

                /// Address of the sequence of blocks.
                ptr: *mut u8,

                /// Length in bytes of the sequence of blocks.
                ///
                /// This length must be dividable by 16.
                len: usize,
            } -> ()
        },
    ];
    Item::Mod(Mod { docs, name, items })
}
