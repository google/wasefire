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

#[derive(Default)]
pub struct Store {
    names: [Option<String>; MAX_NAME_CNT],
}

impl Store {
    pub fn new() -> Self {
        let mut store = Store::default();
        for (i, slot) in store.names.iter_mut().enumerate() {
            if let Some(name) = store::find(2 * i).unwrap() {
                *slot = Some(String::from_utf8(name.into()).unwrap());
            }
        }
        store
    }

    pub fn insert(&mut self, name: String, private: [u8; 32]) -> Result<(), String> {
        if MAX_NAME_LEN < name.len() {
            Err("name too long")?;
        }
        let mut free = None;
        for (i, slot) in self.names.iter_mut().enumerate() {
            match slot {
                None if free.is_none() => free = Some(i),
                Some(x) if *x == name => Err("name already registered")?,
                _ => (),
            }
        }
        let i = free.ok_or("too many registered names")?;
        store::insert(2 * i + 1, &private).unwrap();
        store::insert(2 * i, name.as_bytes()).unwrap();
        self.names[i] = Some(name);
        Ok(())
    }

    pub fn find(&self, name: &str) -> Result<[u8; 32], String> {
        let private = match self.names.iter().position(|x| x.as_deref() == Some(name)) {
            None => Err("name not registered")?,
            Some(i) => store::find(2 * i + 1).unwrap().ok_or("corrupted store")?,
        };
        Ok(*<&[u8; 32]>::try_from(&private[..]).map_err(|_| "corrupted store")?)
    }

    pub fn list(&self) -> Vec<String> {
        self.names.iter().filter_map(|x| x.clone()).collect()
    }

    pub fn delete(&mut self, name: &str) -> Result<(), String> {
        let i = match self.names.iter_mut().position(|x| x.as_deref() == Some(name)) {
            Some(x) => x,
            None => Err("name not registered")?,
        };
        self.names[i] = None;
        store::remove(2 * i).unwrap();
        store::remove(2 * i + 1).unwrap();
        Ok(())
    }
}

/// Maximum number of names.
const MAX_NAME_CNT: usize = 5;

/// Maximum length in bytes of a name.
const MAX_NAME_LEN: usize = 20;
