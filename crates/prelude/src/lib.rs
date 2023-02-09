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

//! This crate provides high-level access to the applet API.
//!
//! In particular, it provides the following:
//! - A panic handler that prints the panic info and traps.
//! - A global allocator using the [rlsf] crate.
//! - High-level wrappers around the applet API.
//!
//! The high-level wrappers provide the following:
//! - A safe API. (The applet API requires `unsafe`.)
//! - Rust types like byte slices (instead of their internal representation).
//! - Closures for callbacks.
//!
//! [rlsf]: https://crates.io/crates/rlsf

#![no_std]
#![feature(alloc_error_handler)]
#![feature(macro_metavar_expr)]
#![feature(vec_into_raw_parts)]

extern crate alloc;

mod allocator;
pub mod button;
mod callback;
pub mod clock;
pub mod crypto;
pub mod debug;
pub mod led;
pub mod rng;
pub mod scheduling;
pub mod store;
pub mod usb;

pub use debug::println;

#[panic_handler]
fn handle_panic(info: &core::panic::PanicInfo) -> ! {
    println!("{}", info);
    core::arch::wasm32::unreachable()
}
