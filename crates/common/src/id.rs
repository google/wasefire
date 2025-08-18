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

use wasefire_sync::{AtomicU32, Ordering};

/// Returns numbers from 1 to u32::MAX (inclusive).
#[derive(Default)]
pub struct UniqueId {
    count: AtomicU32,
}

impl UniqueId {
    /// Creates a new sequence.
    pub const fn new() -> Self {
        Self { count: AtomicU32::new(0) }
    }

    /// Gets the next element in the sequence.
    ///
    /// # Panics
    ///
    /// Panics if called more than `u32::MAX` times on a given sequence.
    pub fn next(&self) -> u32 {
        let result = self.count.fetch_add(1, Ordering::Relaxed);
        assert!(result < u32::MAX);
        result + 1
    }
}
