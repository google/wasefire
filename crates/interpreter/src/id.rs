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

use atomic_polyfill::{AtomicUsize, Ordering};

/// Returns numbers from 1 to u32::MAX (inclusive).
pub struct UniqueId {
    count: AtomicUsize,
}

impl UniqueId {
    pub const fn new() -> Self {
        Self { count: AtomicUsize::new(0) }
    }

    pub fn next(&self) -> usize {
        let result = self.count.fetch_add(1, Ordering::SeqCst);
        assert!(result < u32::MAX as usize);
        result + 1
    }
}
