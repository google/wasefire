// Copyright 2026 Google LLC
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

//! Persistent store interface.

use alloc::vec::Vec;

use wasefire_error::Code;

use crate::Error;

#[cfg(feature = "api-store-fragment")]
pub mod fragment;

/// Persistent store interface.
pub trait Api: Send {
    /// Fragmented entries interface.
    #[cfg(feature = "api-store-fragment")]
    type Fragment: fragment::Api;

    /// Maximum valid key.
    const MAX_KEY: usize;

    /// Maximum valid value length.
    ///
    /// For longer values, the [`Fragment`] API must be used.
    const MAX_LEN: usize;

    /// Inserts an entry.
    fn insert(key: usize, value: &[u8]) -> Result<(), Error>;

    /// Removes an entry.
    fn remove(key: usize) -> Result<(), Error>;

    /// Finds an entry.
    fn find(key: usize) -> Result<Option<Vec<u8>>, Error>;

    /// Returns the keys of all entries.
    fn keys() -> Result<Vec<u16>, Error>;

    /// Removes all entries between `min_key` and `MAX_KEY`.
    fn clear(min_key: usize) -> Result<(), Error>;
}

/// Fragmented entries interface.
#[cfg(feature = "api-store-fragment")]
pub type Fragment<B> = <super::Store<B> as Api>::Fragment;

/// Helper trait for boards using the `wasefire-store` crate.
pub trait HasStore: Send {
    /// Underlying storage.
    type Storage: wasefire_store::Storage;

    /// Number of keys reserved for the board.
    const FIRST_KEY: usize = 0;

    /// Provides scoped access to the store.
    fn with_store<R>(f: impl FnOnce(&mut wasefire_store::Store<Self::Storage>) -> R) -> R;
}

/// Wrapper type for boards using the `wasefire-store` crate.
pub struct WithStore<T: HasStore> {
    _never: !,
    _has_store: T,
}

fn shift_key<T: HasStore>(key: usize) -> Result<usize, Error> {
    T::FIRST_KEY.checked_add(key).ok_or(Error::user(Code::InvalidArgument))
}

impl<T: HasStore> Api for WithStore<T> {
    #[cfg(feature = "api-store-fragment")]
    type Fragment = WithStore<T>;

    const MAX_KEY: usize = wasefire_store::format::MAX_KEY_INDEX as usize - T::FIRST_KEY;
    const MAX_LEN: usize = wasefire_store::format::MAX_VALUE_LEN as usize;

    fn insert(key: usize, value: &[u8]) -> Result<(), Error> {
        let key = shift_key::<T>(key)?;
        T::with_store(|store| store.insert(key, value))
    }

    fn remove(key: usize) -> Result<(), Error> {
        let key = shift_key::<T>(key)?;
        T::with_store(|store| store.remove(key))
    }

    fn find(key: usize) -> Result<Option<Vec<u8>>, Error> {
        let key = shift_key::<T>(key)?;
        T::with_store(|store| store.find(key))
    }

    fn keys() -> Result<Vec<u16>, Error> {
        T::with_store(|store| {
            let mut keys = Vec::new();
            for handle in store.iter()? {
                if let Some(key) = handle?.get_key().checked_sub(T::FIRST_KEY) {
                    keys.push(key as u16);
                }
            }
            Ok(keys)
        })
    }

    fn clear(min_key: usize) -> Result<(), Error> {
        let min_key = shift_key::<T>(min_key)?;
        T::with_store(|store| store.clear(min_key))
    }
}
