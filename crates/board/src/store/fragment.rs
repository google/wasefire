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

//! Fragmented entries interface.

use alloc::vec::Vec;
use core::ops::Range;

use wasefire_store::fragment;

use crate::Error;

/// Fragmented entries interface.
pub trait Api: Send {
    /// Reads a fragmented entry.
    fn read(keys: Range<usize>) -> Result<Option<Vec<u8>>, Error>;

    /// Writes a fragmented entry.
    fn write(keys: Range<usize>, value: &[u8]) -> Result<(), Error>;

    /// Deletes a fragmented entry.
    fn delete(keys: Range<usize>) -> Result<(), Error>;
}

impl<T: super::HasStore> Api for super::WithStore<T> {
    fn read(keys: Range<usize>) -> Result<Option<Vec<u8>>, Error> {
        let start = super::shift_key::<T>(keys.start)?;
        let end = super::shift_key::<T>(keys.end)?;
        T::with_store(|store| fragment::read(store, &(start .. end)))
    }

    fn write(keys: Range<usize>, value: &[u8]) -> Result<(), Error> {
        let start = super::shift_key::<T>(keys.start)?;
        let end = super::shift_key::<T>(keys.end)?;
        T::with_store(|store| fragment::write(store, &(start .. end), value))
    }

    fn delete(keys: Range<usize>) -> Result<(), Error> {
        let start = super::shift_key::<T>(keys.start)?;
        let end = super::shift_key::<T>(keys.end)?;
        T::with_store(|store| fragment::delete(store, &(start .. end)))
    }
}
