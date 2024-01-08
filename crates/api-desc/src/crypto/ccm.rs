// Copyright 2022 Google LLC
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
        /// AES-CCM according to Bluetooth.
    };
    let name = "ccm".into();
    let items = vec![
        item! {
            /// Whether AES-CCM is supported.
            ///
            /// On success, returns 1 if supported, 0 otherwise.
            fn is_supported "ccs" {}
        },
        item! {
            /// Encrypts a clear text given a key and IV.
            ///
            /// Returns zero on success.
            fn encrypt "cce" {
                /// The 16 bytes key to encrypt with.
                key: *const u8,

                /// The 8 bytes IV to encrypt with.
                iv: *const u8,

                /// Length in bytes of the `clear` text.
                ///
                /// This must be at most 251 bytes. The `cipher` length must be 4 bytes longer than
                /// this value.
                len: usize,

                /// The clear text to encrypt from.
                ///
                /// Its length must be provided in the `len` field.
                clear: *const u8,

                /// The cipher text to encrypt to.
                ///
                /// Its length must be `len + 4` bytes.
                cipher: *mut u8,
            }
        },
        item! {
            /// Decrypts a cipher text given a key and IV.
            ///
            /// Returns zero on success.
            fn decrypt "ccd" {
                /// The 16 bytes key to encrypt with.
                key: *const u8,

                /// The 8 bytes IV to encrypt with.
                iv: *const u8,

                /// Length in bytes of the `clear` text.
                ///
                /// This must be at most 251 bytes. The `cipher` length must be 4 bytes longer than
                /// this value.
                len: usize,

                /// The cipher text to encrypt from.
                ///
                /// Its length must be `len + 4` bytes.
                cipher: *const u8,

                /// The clear text to encrypt to.
                ///
                /// Its length must be provided in the `len` field.
                clear: *mut u8,
            }
        },
    ];
    Item::Mod(Mod { docs, name, items })
}
