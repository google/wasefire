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

//! Provides a mutable lazy cell.
//!
//! This should eventually be superseded by `std::cell::LazyCell`.

use anyhow::Result;

pub enum Lazy<T, F: FnOnce() -> Result<T>> {
    Uninit(F),
    Init(T),
    Empty,
}

impl<T, F: FnOnce() -> Result<T>> Lazy<T, F> {
    pub fn new(init: F) -> Self {
        Lazy::Uninit(init)
    }

    pub fn get(&mut self) -> Result<&mut T> {
        if let Lazy::Init(x) = self {
            return Ok(x);
        }
        match std::mem::replace(self, Lazy::Empty) {
            Lazy::Uninit(f) => *self = Lazy::Init(f()?),
            _ => unreachable!(),
        };
        self.get()
    }
}
