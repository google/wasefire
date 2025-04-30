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

use core::ops::Range;

use derive_where::derive_where;

use crate::toctou::*;

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub struct CursorState {
    start: usize,
    end: usize,
}

impl CursorState {
    pub fn from_range(range: Range<usize>) -> Self {
        CursorState { start: range.start, end: range.end }
    }

    pub fn start(self) -> usize {
        self.start
    }

    pub fn end(self) -> usize {
        self.end
    }

    fn new(len: usize) -> Self {
        CursorState { start: 0, end: len }
    }

    fn is_empty(self) -> bool {
        self.end == self.start
    }

    fn len(self) -> usize {
        unwrap(self.end.checked_sub(self.start))
    }

    fn range(self) -> Range<usize> {
        self.start .. self.end
    }
}

#[derive(Debug)]
#[derive_where(Default, Clone)]
pub struct Cursor<'a, T> {
    slice: &'a [T],
    state: CursorState, // within bounds
}

impl<'a, T> Cursor<'a, T> {
    pub fn new(slice: &'a [T]) -> Self {
        Cursor { slice, state: CursorState::new(slice.len()) }
    }

    pub fn shrink(&mut self) {
        self.slice = &self.slice[self.state.range()];
        self.state.start = 0;
        self.state.end = self.slice.len();
    }

    pub fn save(&self) -> CursorState {
        self.state
    }

    // Safety: Must be a previously saved state.
    pub unsafe fn restore(&mut self, state: CursorState) {
        self.state = state;
    }

    pub fn is_empty(&self) -> bool {
        self.state.is_empty()
    }

    pub fn get(&self, index: usize) -> &'a T {
        unwrap(unwrap(self.slice.get(self.state.range())).get(index))
    }

    pub fn split<M: Mode>(&mut self, len: usize) -> MResult<Cursor<'a, T>, M> {
        M::check(|| len <= self.state.len())?;
        let mut result = self.clone();
        #[cfg(feature = "toctou")]
        (self.state.start = self.state.start.strict_add(len));
        #[cfg(not(feature = "toctou"))]
        (self.state.start = unsafe { self.state.start.unchecked_add(len) });
        result.state.end = self.state.start;
        Ok(result)
    }

    pub fn consume(self) -> &'a [T] {
        unwrap(self.slice.get(self.state.range()))
    }

    pub fn adjust_start(&mut self, off: isize) {
        let new_start = self.state.start.strict_add_signed(off);
        assert!(new_start <= self.state.end);
        self.state.start = new_start;
    }
}
