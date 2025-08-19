// Copyright 2025 Google LLC
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

//! Slice with dynamic borrow checking.

#![no_std]
#![cfg_attr(feature = "_internal", feature(array_windows))]
#![feature(slice_ptr_get)]

extern crate alloc;

use core::cell::RefCell;
use core::marker::PhantomData;
use core::ops::{Range, RangeBounds};

#[cfg(feature = "_internal")]
pub use internal::*;

/// Slice with dynamic borrow checking.
pub struct SliceCell<'a, T> {
    _lifetime: PhantomData<&'a ()>,
    data: *mut [T],
    state: RefCell<internal::State>,
}

/// Access errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    /// Invalid range.
    Range,

    /// Invalid borrow.
    Borrow,
}

impl<'a, T> SliceCell<'a, T> {
    /// Returns the same exclusive slice but with dynamic borrow checking.
    pub fn new(data: &'a mut [T]) -> Self {
        Self { _lifetime: PhantomData, data, state: Default::default() }
    }

    /// Returns a shared reference to an element of the underlying slice.
    pub fn get(&self, index: usize) -> Result<&T, Error> {
        Ok(&self.get_range(index ..= index)?[0])
    }

    /// Returns an exclusive reference to an element of the underlying slice.
    pub fn get_mut(&self, index: usize) -> Result<&mut T, Error> {
        Ok(&mut self.get_range_mut(index ..= index)?[0])
    }

    /// Returns a shared slice reference of the underlying slice.
    pub fn get_range(&self, range: impl RangeBounds<usize>) -> Result<&[T], Error> {
        let range = self.range(range)?;
        let false = range.is_empty() else { return Ok(&[]) };
        // SAFETY: Checked by self.range() above.
        let ptr = unsafe { self.data.as_mut_ptr().add(range.start) };
        let len = range.len();
        self.borrow(range)?;
        // SAFETY: Checked by self.borrow() above.
        Ok(unsafe { core::slice::from_raw_parts(ptr, len) })
    }

    /// Returns an exclusive slice reference of the underlying slice.
    #[allow(clippy::mut_from_ref)]
    pub fn get_range_mut(&self, range: impl RangeBounds<usize>) -> Result<&mut [T], Error> {
        let range = self.range(range)?;
        let false = range.is_empty() else { return Ok(&mut []) };
        // SAFETY: Checked by self.range() above.
        let ptr = unsafe { self.data.as_mut_ptr().add(range.start) };
        let len = range.len();
        self.borrow_mut(range)?;
        // SAFETY: Checked by self.borrow_mut() above.
        Ok(unsafe { core::slice::from_raw_parts_mut(ptr, len) })
    }

    /// Invalidates all references to the underlying slice.
    pub fn reset(&mut self) {
        self.state.take();
    }

    fn range(&self, range: impl RangeBounds<usize>) -> Result<Range<usize>, Error> {
        internal::range_check(self.data.len(), range)
    }

    fn borrow(&self, range: Range<usize>) -> Result<(), Error> {
        let access = internal::Access { exclusive: false, range };
        internal::borrow_check(&mut self.state.borrow_mut(), access)
    }

    fn borrow_mut(&self, range: Range<usize>) -> Result<(), Error> {
        let access = internal::Access { exclusive: true, range };
        internal::borrow_check(&mut self.state.borrow_mut(), access)
    }
}

#[cfg_attr(not(feature = "_internal"), allow(unreachable_pub))]
mod internal {
    use alloc::vec::Vec;
    use core::ops::{Bound, Range, RangeBounds};

    use crate::Error;

    /// Makes sure the range is a sub-range of the underlying slice.
    pub fn range_check(len: usize, range: impl RangeBounds<usize>) -> Result<Range<usize>, Error> {
        let start = match range.start_bound() {
            Bound::Included(x) => *x,
            Bound::Excluded(x) => x.checked_add(1).ok_or(Error::Range)?,
            Bound::Unbounded => 0,
        };
        let end = match range.end_bound() {
            Bound::Included(x) => x.checked_add(1).ok_or(Error::Range)?,
            Bound::Excluded(x) => *x,
            Bound::Unbounded => len,
        };
        if start <= end && end <= len { Ok(start .. end) } else { Err(Error::Range) }
    }

    /// Sorted list of non-overlapping non-empty accesses since the last reset.
    pub type State = Vec<Access>;

    /// Describes an access into a slice.
    #[cfg_attr(feature = "_internal", derive(Clone, PartialEq, Eq))]
    pub struct Access {
        /// Whether the access is shared or exclusive.
        pub exclusive: bool,
        /// The non-empty range that is accessed.
        pub range: Range<usize>,
    }

    #[cfg(feature = "_internal")]
    impl core::fmt::Debug for Access {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            write!(f, "{}{:?}", if self.exclusive { "mut " } else { "" }, self.range)
        }
    }

    /// Checks whether the state invariant holds.
    #[cfg(feature = "_internal")]
    pub fn state_invariant(state: &State) -> bool {
        state.array_windows().all(|[prev, next]| prev.range.end <= next.range.start)
    }

    /// Makes sure the non-empty range can be borrowed (shared or exclusively).
    ///
    /// If the range can be borrowed, the state is updated to reflect that access.
    pub fn borrow_check(state: &mut State, new: Access) -> Result<(), Error> {
        debug_assert!(!new.range.is_empty());
        // Find the first existing access that ends after the new access starts.
        let Some(i) = state.iter().position(|cur| new.range.start < cur.range.end) else {
            // The new access does not overlap and is after all existing accesses.
            state.push(new);
            return Ok(());
        };
        // Find the first existing access that starts after the new access ends.
        let j = match state[i ..].iter().position(|cur| new.range.end <= cur.range.start) {
            None => state.len(),
            Some(x) => i + x,
        };
        if i == j {
            // The new access does not overlap and is before an existing access.
            state.insert(i, new);
            return Ok(());
        }
        // The new access overlaps with the existing accesses between i and j.
        if new.exclusive || state[i .. j].iter().any(|x| x.exclusive) {
            // Either the new or at least one of the existing accesses is exclusive.
            return Err(Error::Borrow);
        }
        // Merge the new access with all the existing overlapping ones.
        state[i].range.start = core::cmp::min(state[i].range.start, new.range.start);
        state[i].range.end = core::cmp::max(state[j - 1].range.end, new.range.end);
        state.drain(i + 1 .. j);
        Ok(())
    }
}
