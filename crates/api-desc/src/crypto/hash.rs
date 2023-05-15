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
        /// Hash functions.
    };
    let name = "hash".into();
    let items = vec![
        item! {
            /// Hash algorithm.
            enum Algorithm {
                /// SHA-256.
                Sha256 = 0,

                /// SHA-384.
                Sha384 = 1,
            }
        },
        item! {
            /// Whether the algorithm is supported.
            fn is_supported "chs" {
                /// The hash algorithm.
                algorithm: usize,
            } -> {
                /// 1 if supported, 0 otherwise.
                supported: usize,
            }
        },
        item! {
            /// Initializes a hash.
            fn initialize "chi" {
                /// The hash algorithm.
                algorithm: usize,
            } -> {
                /// A non-negative identifier on success, bitwise complement of
                /// [`Error`](crate::crypto::Error) otherwise.
                id: isize,
            }
        },
        item! {
            /// Updates a hash.
            ///
            /// Errors are surfaced in the [`finalize()`] call.
            fn update "chu" {
                /// The identifier returned by the associated [`initialize()`] call.
                id: usize,

                /// The pointer to the data to hash.
                data: *const u8,

                /// The length of the data to hash.
                length: usize,
            } -> {}
        },
        item! {
            /// Finalizes a hash.
            fn finalize "chf" {
                /// The identifier returned by the associated [`initialize()`] call.
                ///
                /// This is consumed and invalidated by this call regardless of the return value.
                id: usize,

                /// The pointer to the buffer where the digest must be written.
                ///
                /// Its length is defined by the algorithm:
                /// - 32 bytes for SHA-256.
                ///
                /// The pointer may be null, in which case this function deallocates the identifier
                /// without computing the digest.
                digest: *mut u8,
            } -> {
                /// Zero on success, bitwise complement of [`Error`](crate::crypto::Error)
                /// otherwise.
                res: isize,
            }
        },
        item! {
            /// Whether the algorithm is supported for hmac.
            fn is_hmac_supported "cht" {
                /// The hash algorithm.
                algorithm: usize,
            } -> {
                /// 1 if supported, 0 otherwise.
                supported: usize,
            }
        },
        item! {
            /// Initializes an hmac.
            fn hmac_initialize "chj" {
                /// The hash algorithm.
                algorithm: usize,

                /// The pointer to the key.
                key: *const u8,

                /// The length of the key.
                ///
                /// If greater than 64 bytes, the key will be itself hashed.
                key_len: usize,
            } -> {
                /// A non-negative identifier on success, bitwise complement of
                /// [`Error`](crate::crypto::Error) otherwise.
                id: isize,
            }
        },
        item! {
            /// Updates an hmac.
            ///
            /// Errors are surfaced in the [`hmac_finalize()`] call.
            fn hmac_update "chv" {
                /// The identifier returned by the associated [`hmac_initialize()`] call.
                id: usize,

                /// The pointer to the data to hmac.
                data: *const u8,

                /// The length of the data to hmac.
                length: usize,
            } -> {}
        },
        item! {
            /// Finalizes an hmac.
            fn hmac_finalize "chg" {
                /// The identifier returned by the associated [`hmac_initialize()`] call.
                ///
                /// This is consumed and invalidated by this call regardless of the return value.
                id: usize,

                /// The pointer to the buffer where the hmac must be written.
                ///
                /// Its length is defined by the algorithm:
                /// - 32 bytes for SHA-256.
                ///
                /// The pointer may be null, in which case this function deallocates the identifier
                /// without computing the hmac.
                hmac: *mut u8,
            } -> {
                /// Zero on success, bitwise complement of [`Error`](crate::crypto::Error)
                /// otherwise.
                res: isize,
            }
        },
        item! {
            /// Whether the algorithm is supported for hkdf.
            fn is_hkdf_supported "chr" {
                /// The hash algorithm.
                algorithm: usize,
            } -> {
                /// 1 if supported, 0 otherwise.
                supported: usize,
            }
        },
        item! {
            /// Expands with RFC5869 HKDF.
            fn hkdf_expand "che" {
                /// The hash algorithm.
                algorithm: usize,

                /// The pointer to the pseudo random key.
                prk: *const u8,

                /// The length of the pseudo random key.
                ///
                /// Must be at least the length of the hash algorithm output.
                prk_len: usize,

                /// The pointer to the info.
                ///
                /// May be null if [`info_len`] is null.
                info: *const u8,

                /// The length of the info.
                ///
                /// May be zero.
                info_len: usize,

                /// The pointer to the output key material.
                okm: *mut u8,

                /// The length of the output key material.
                ///
                /// Must be at most 255 times the output length of the hash algorithm.
                okm_len: usize,

            } -> {
                /// Zero on success, bitwise complement of [`Error`](crate::crypto::Error)
                /// otherwise.
                res: isize,
            }
        },
    ];
    Item::Mod(Mod { docs, name, items })
}
