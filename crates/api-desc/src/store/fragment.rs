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
        /// Support for fragmented entries.
    };
    let name = "fragment".into();
    let items = vec![
        item! {
            /// Inserts an entry in the store.
            ///
            /// The entry will be fragmented over multiple keys within the provided range as needed.
            ///
            /// If an entry for that range of keys was already present, it is overwritten.
            fn insert "sfi" {
                /// Range of keys where to insert the fragments.
                ///
                /// This is a pair of u16: the lowest u16 is the first key of the range and the
                /// highest u16 is one past the last key of the range.
                keys: u32,

                /// Value of the entry.
                ptr: *const u8,

                /// Length of the value.
                len: usize,
            } -> ()
        },
        item! {
            /// Removes an entry from the store.
            ///
            /// All fragments from the range of keys will be deleted.
            ///
            /// This is not an error if no entry is present. This is simply a no-op in that case.
            fn remove "sfr" {
                /// Range of keys to remove.
                keys: u32,
            } -> ()
        },
        item! {
            /// Finds an entry in the store, if any.
            ///
            /// The entry may be fragmented withen the provided range.
            ///
            /// Returns whether an entry was found.
            ///
            /// This is an [allocating function](crate#allocating-memory).
            fn find "sff" {
                /// Range of keys to concatenate as an entry.
                keys: u32,

                /// Where to write the value of the entry, if found.
                ptr: *mut *mut u8,

                /// Where to write the length of the value, if found.
                len: *mut usize,
            } -> bool
        },
    ];
    Item::Mod(Mod { docs, name, items })
}
