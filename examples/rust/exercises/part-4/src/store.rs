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

use alloc::string::String;
use alloc::vec::Vec;

use wasefire::store;

// TODO: The mapping from name to private key is stored as an associative list where the name is
// store at the storage key `2 * k` and the private key at `2 * k + 1` where `k < MAX_NAME_CNT`.
#[derive(Default)]
pub struct Store {
    // TODO(optional): You may implement a cache.
    // /// Cache mapping from storage key to private key name.
    // names: [Option<String>; MAX_NAME_CNT],
}

impl Store {
    pub fn new() -> Self {
        let mut store = Store::default();
        // TODO(optional): Populate the cache. Use store::find() to read an storage entry and
        // String::from_utf8() to convert names.
        store
    }

    pub fn insert(&mut self, name: String, private: [u8; 32]) -> Result<(), String> {
        // TODO: Return an error if the name is longer than MAX_NAME_LEN.
        // TODO: Loop through all name entries to find an available slot.
        // TODO: Return an error if the name is already present.
        // TODO: Return an error if there are no available slot.
        // TODO: Use store::insert() to store both the name and private key at their appropriate
        // storage key.
        // TODO(optional): Update the cache.
        todo!()
    }

    pub fn find(&self, name: &str) -> Result<[u8; 32], String> {
        // TODO: Find the entry where the name is stored (optional: using the cache).
        // TODO: Return an error if the name is not registered.
        // TODO: Read the private key using store::find() on the next storage key.
        // TODO: Return an error if there's no associated private key.
        // TODO: Use <&[u8; 32]>::try_from() to convert the private key.
        // TODO: Return an error if the conversion failed.
        todo!()
    }

    pub fn list(&self) -> Vec<String> {
        // TODO(optional): Use the cache.
        todo!()
    }

    pub fn delete(&mut self, name: &str) -> Result<(), String> {
        // TODO: Find the entry where the name is stored (optional: using the cache).
        // TODO: Return an error if the name is not registered.
        // TODO: Remove the name and private key entries using store::remove().
        // TODO(optional): Update the cache.
        todo!()
    }
}

/// Maximum number of names.
const MAX_NAME_CNT: usize = 5;

/// Maximum length in bytes of a name.
const MAX_NAME_LEN: usize = 20;
