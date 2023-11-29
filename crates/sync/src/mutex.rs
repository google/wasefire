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

use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};

use portable_atomic::AtomicBool;
use portable_atomic::Ordering::{Acquire, Relaxed, Release};

/// Non-blocking non-reentrant mutex.
///
/// Locking this mutex will panic if it is already locked. In particular, it will not block.
pub struct Mutex<T> {
    lock: AtomicBool,
    data: UnsafeCell<T>,
}

/// Locks a mutex and provides access to its content until dropped.
pub struct MutexGuard<'a, T>(&'a Mutex<T>);

unsafe impl<T: Send> Sync for Mutex<T> {}

impl<T> Mutex<T> {
    /// Creates a new mutex.
    pub const fn new(data: T) -> Self {
        Mutex { lock: AtomicBool::new(false), data: UnsafeCell::new(data) }
    }

    /// Tries to lock the mutex.
    pub fn try_lock(&self) -> Option<MutexGuard<'_, T>> {
        self.lock.compare_exchange(false, true, Acquire, Relaxed).ok()?;
        Some(MutexGuard(self))
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
}

impl<'a, T> Deref for MutexGuard<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.0.data.get() }
    }
}

impl<'a, T> DerefMut for MutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.0.data.get() }
    }
}

impl<'a, T> Drop for MutexGuard<'a, T> {
    fn drop(&mut self) {
        self.0.lock.store(false, Release);
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
