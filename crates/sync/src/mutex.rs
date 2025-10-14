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

/// Non-blocking non-reentrant mutex.
///
/// Locking this mutex will panic if it is already locked. In particular, it will not block.
pub struct Mutex<T>(spin::Mutex<T>);

/// Locks a mutex and provides access to its content until dropped.
pub type MutexGuard<'a, T> = spin::MutexGuard<'a, T>;

impl<T> Mutex<T> {
    /// Creates a new mutex.
    pub const fn new(data: T) -> Self {
        Mutex(spin::Mutex::new(data))
    }

    /// Tries to lock the mutex.
    pub fn try_lock(&self) -> Option<MutexGuard<'_, T>> {
        self.0.try_lock()
    }

    /// Locks the mutex.
    ///
    /// # Panics
    ///
    /// Panics if it is already locked.
    #[track_caller]
    pub fn lock(&self) -> MutexGuard<'_, T> {
        self.try_lock().unwrap()
    }

    /// Consumes the mutex.
    pub fn into_inner(self) -> T {
        self.0.into_inner()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn static_mutex() {
        static MUTEX: Mutex<i32> = Mutex::new(42);
        *MUTEX.lock() = 13;
        assert_eq!(*MUTEX.lock(), 13);
    }

    #[test]
    #[should_panic]
    fn double_lock() {
        let mutex = Mutex::new(42);
        let _guard = mutex.lock();
        mutex.lock();
    }
}
