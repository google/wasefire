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

//! Provides API for mutexes and atomics.
//!
//! This module is currently only correct when compiled without the atomics proposal.
#![cfg(not(target_feature = "atomics"))]

use core::cell::{Cell, RefCell, RefMut};

pub use portable_atomic::*;

pub struct Mutex<T: ?Sized> {
    locked: Cell<bool>,
    data: RefCell<T>,
}

pub struct MutexGuard<'a, T: ?Sized + 'a> {
    locked: &'a Cell<bool>,
    data: RefMut<'a, T>,
}

// SAFETY: This is a single-threaded environment. The bound on Send comes from std::sync::Mutex, not
// sure if relevant or needed, but it's safer to have it.
unsafe impl<T: ?Sized + Send> Sync for Mutex<T> {}

impl<T> Mutex<T> {
    pub const fn new(value: T) -> Self {
        Self { locked: Cell::new(false), data: RefCell::new(value) }
    }

    pub fn lock(&self) -> MutexGuard<T> {
        self.try_lock().expect("cannot recursively acquire mutex")
    }

    pub fn try_lock(&self) -> Option<MutexGuard<T>> {
        if self.locked.replace(true) {
            return None;
        }
        Some(MutexGuard { locked: &self.locked, data: self.data.borrow_mut() })
    }
}

impl<T: ?Sized> core::ops::Deref for MutexGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.data.deref()
    }
}

impl<T: ?Sized> core::ops::DerefMut for MutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.data.deref_mut()
    }
}

impl<T: ?Sized> Drop for MutexGuard<'_, T> {
    fn drop(&mut self) {
        self.locked.set(false);
    }
}
