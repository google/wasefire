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
        /// Persistent storage operations.
    };
    let name = "store".into();
    let items = vec![
        item! {
            /// Describes errors interacting with the store.
            enum Error {
                /// A function pre-condition was broken.
                InvalidArgument = 0,

                /// The store is full.
                NoCapacity = 1,

                /// The store reached its end of life.
                NoLifetime = 2,

                /// An operation to the underlying storage failed.
                StorageError = 3,

                /// The underlying storage doesn't match the store invariant.
                InvalidStorage = 4,
            }
        },
        item! {
            /// Inserts an entry in the store.
            ///
            /// If an entry for that key was already present, it is overwritten.
            fn insert "si" {
                /// Key of the entry.
                ///
                /// This must be smaller than 4096.
                key: usize,

                /// Value of the entry.
                ptr: *const u8,

                /// Length of the value.
                len: usize,
            } -> {
                /// Zero for success. Otherwise complement of error number.
                res: isize,
            }
        },
        item! {
            /// Removes an entry from the store.
            ///
            /// This is not an error if no entry is present. This is simply a no-op in that case.
            fn remove "sr" {
                /// Key of the entry.
                key: usize,
            } -> {
                /// Zero for success. Otherwise complement of error number.
                res: isize,
            }
        },
        item! {
            /// Finds an entry in the store, if any.
            fn find "sf" {
                /// Key of the entry to find.
                key: usize,

                /// Where to write the value of the entry, if found.
                ///
                /// The (inner) pointer will be allocated by the callee and must be freed by the
                /// caller. It is thus owned by the caller when the function returns.
                #[cfg(not(feature = "multivalue"))]
                ptr: *mut *mut u8,

                /// Where to write the length of the value, if found.
                #[cfg(not(feature = "multivalue"))]
                len: *mut usize,
            } -> {
                /// Value of the entry if found. Null if not found.
                ///
                /// The pointer is allocated by the callee and must be freed by the caller. It is
                /// thus owned by the caller when the function returns.
                #[cfg(feature = "multivalue")]
                ptr: *mut u8,

                /// Length of the value if non-negative. Otherwise complement of error number.
                #[cfg(feature = "multivalue")]
                len: isize,

                /// One if found. Zero if not found. Otherwise complement of error number.
                #[cfg(not(feature = "multivalue"))]
                res: isize,
            }
        },
    ];
    Item::Mod(Mod { docs, name, items })
}
