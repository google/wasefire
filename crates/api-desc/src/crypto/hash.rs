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
        #[cfg(feature = "api-crypto-hash")]
        item! {
            /// Whether the algorithm is supported.
            ///
            /// On success, returns 1 if supported, 0 otherwise.
            fn is_supported "chs" {
                /// The hash algorithm.
                algorithm: usize,
            }
        },
        #[cfg(feature = "api-crypto-hash")]
        item! {
            /// Initializes a hash.
            ///
            /// Returns the hash identifier on success.
            fn initialize "chi" {
                /// The hash algorithm.
                algorithm: usize,
            }
        },
        #[cfg(feature = "api-crypto-hash")]
        item! {
            /// Updates a hash.
            ///
            /// Errors are surfaced in the [`finalize()`] call.
            ///
            /// Returns zero on success.
            fn update "chu" {
                /// The identifier returned by the associated [`initialize()`] call.
                id: usize,

                /// The pointer to the data to hash.
                data: *const u8,

                /// The length of the data to hash.
                length: usize,
            }
        },
        #[cfg(feature = "api-crypto-hash")]
        item! {
            /// Finalizes a hash.
            ///
            /// Returns zero on success.
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
            }
        },
        #[cfg(feature = "api-crypto-hmac")]
        item! {
            /// Whether the algorithm is supported for hmac.
            ///
            /// On success, returns 1 if supported, 0 otherwise.
            fn is_hmac_supported "cht" {
                /// The hash algorithm.
                algorithm: usize,
            }
        },
        #[cfg(feature = "api-crypto-hmac")]
        item! {
            /// Initializes an hmac.
            ///
            /// Returns the hmac identifier on success.
            fn hmac_initialize "chj" {
                /// The hash algorithm.
                algorithm: usize,

                /// The pointer to the key.
                key: *const u8,

                /// The length of the key.
                ///
                /// If greater than 64 bytes, the key will be itself hashed.
                key_len: usize,
            }
        },
        #[cfg(feature = "api-crypto-hmac")]
        item! {
            /// Updates an hmac.
            ///
            /// Errors are surfaced in the [`hmac_finalize()`] call.
            ///
            /// Returns zero on success.
            fn hmac_update "chv" {
                /// The identifier returned by the associated [`hmac_initialize()`] call.
                id: usize,

                /// The pointer to the data to hmac.
                data: *const u8,

                /// The length of the data to hmac.
                length: usize,
            }
        },
        #[cfg(feature = "api-crypto-hmac")]
        item! {
            /// Finalizes an hmac.
            ///
            /// Returns zero on success.
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
            }
        },
        #[cfg(feature = "api-crypto-hkdf")]
        item! {
            /// Whether the algorithm is supported for hkdf.
            ///
            /// On success, returns 1 if supported, 0 otherwise.
            fn is_hkdf_supported "chr" {
                /// The hash algorithm.
                algorithm: usize,
            }
        },
        #[cfg(feature = "api-crypto-hkdf")]
        item! {
            /// Expands with RFC5869 HKDF.
            ///
            /// Returns zero on success.
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

            }
        },
    ];
    Item::Mod(Mod { docs, name, items })
}
