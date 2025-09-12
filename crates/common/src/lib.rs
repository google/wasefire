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

//! Common Wasefire items.

#![no_std]

pub mod id;
pub mod platform;
pub mod ptr;

/// Returns the address of a linker symbol.
#[macro_export]
macro_rules! addr_of_symbol {
    ($sym:ident) => {{
        unsafe extern "C" {
            static mut $sym: [u8; 0];
        }
        (&raw mut $sym).addr()
    }};
}

/// Extracts a value conditionally.
///
/// The implementation unconditionally calls `take(x)`, so the temporary default value is dropped
/// when `None` is returned.
pub fn take_if<T: Default, R>(x: &mut T, p: impl FnOnce(T) -> Result<R, T>) -> Option<R> {
    match p(core::mem::take(x)) {
        Ok(r) => Some(r),
        Err(e) => {
            *x = e;
            None
        }
    }
}
