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

use crate::Mutex;

/// Convenient wrapper around `Mutex<Option<T>>`.
pub struct TakeCell<T>(Mutex<Option<T>>);

impl<T> TakeCell<T> {
    /// Creates a new mutex-protected option.
    pub const fn new(init: Option<T>) -> Self {
        TakeCell(Mutex::new(init))
    }

    /// Takes the content of the option.
    ///
    /// # Panics
    ///
    /// Panics if the mutex is locked or the option is empty.
    #[track_caller]
    pub fn take(&self) -> T {
        self.0.lock().take().unwrap()
    }

    /// Replaces the content of the option.
    ///
    /// # Panics
    ///
    /// Panics if the mutex is locked.
    #[track_caller]
    pub fn replace(&self, value: T) -> Option<T> {
        self.0.lock().replace(value)
    }

    /// Sets the content of the option.
    ///
    /// # Panics
    ///
    /// Panics if the mutex is locked or the option is not empty.
    #[track_caller]
    pub fn put(&self, value: T) {
        assert!(self.replace(value).is_none())
    }

    /// Executes a closure on the content of the option.
    ///
    /// # Panics
    ///
    /// Panics if the mutex is locked or the option is empty.
    #[track_caller]
    pub fn with<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        f(self.0.lock().as_mut().unwrap())
    }
}
