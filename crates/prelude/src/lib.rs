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
#![feature(doc_cfg)]
#![feature(macro_metavar_expr)]
#![feature(maybe_uninit_array_assume_init)]
#![feature(negative_impls)]
#![feature(never_type)]
#![feature(vec_into_raw_parts)]

extern crate alloc;

pub use wasefire_error::{self as error, Error};
use wasefire_one_of::at_most_one_of;
#[cfg(feature = "rust-crypto")]
use {aead as _, crypto_common as _, digest as _, typenum as _, zeroize as _};

// This internal module is made public because it's used by exported macros called from user code.
#[doc(hidden)]
pub mod internal;

mod allocator;
#[cfg(feature = "api-button")]
pub mod button;
mod callback;
#[cfg(feature = "api-clock")]
pub mod clock;
#[cfg(feature = "internal-api-crypto")]
pub mod crypto;
pub mod debug;
#[cfg(feature = "internal-api-fingerprint")]
pub mod fingerprint;
#[cfg(feature = "api-gpio")]
pub mod gpio;
#[cfg(feature = "api-led")]
pub mod led;
#[cfg(feature = "internal-api-platform")]
pub mod platform;
#[cfg(feature = "api-rng")]
pub mod rng;
#[cfg(feature = "internal-rpc")]
pub mod rpc;
pub mod scheduling;
#[cfg(feature = "internal-serial")]
pub mod serial;
#[cfg(feature = "internal-api-store")]
pub mod store;
pub mod sync;
#[cfg(feature = "api-timer")]
pub mod timer;
#[cfg(feature = "api-uart")]
pub mod uart;
#[cfg(feature = "internal-api-usb")]
pub mod usb;
#[cfg(feature = "api-vendor")]
pub mod vendor;

at_most_one_of!["native", "test", "wasm"];

/// Defines the entry point of an applet.
///
/// This macro brings all items of this crate into scope and makes sure `main()` is the entry point
/// of this applet.
///
/// # Examples
///
/// A typical applet looks like:
///
/// ```ignore
/// #![no_std]
/// wasefire::applet!();
///
/// fn main() {
///     debug!("Hello world!");
/// }
/// ```
#[cfg(not(any(feature = "native", feature = "test")))]
#[macro_export]
macro_rules! applet {
    () => {
        extern crate alloc;

        use wasefire::*;

        #[unsafe(export_name = "main")]
        #[allow(unreachable_code)]
        #[allow(clippy::diverging_sub_expression)]
        extern "C" fn _main() {
            use $crate::internal::Termination as _;
            main().report();
        }
    };
}
#[allow(missing_docs)] // see above
#[cfg(feature = "native")]
#[macro_export]
macro_rules! applet {
    () => {
        extern crate alloc;

        use wasefire::*;

        #[unsafe(no_mangle)]
        #[allow(unreachable_code)]
        #[allow(clippy::diverging_sub_expression)]
        extern "C" fn applet_main() {
            use $crate::internal::Termination as _;
            main().report();
        }
    };
}
#[allow(missing_docs)] // see above
#[cfg(feature = "test")]
#[macro_export]
macro_rules! applet {
    () => {
        extern crate alloc;

        use wasefire::*;
    };
}

#[cfg(not(feature = "test"))]
#[panic_handler]
fn handle_panic(info: &core::panic::PanicInfo) -> ! {
    debug!("{info}");
    scheduling::abort();
}

fn convert(res: isize) -> Result<usize, Error> {
    Error::decode(res as i32).map(|x| x as usize)
}

#[cfg_attr(not(feature = "api-crypto-ec"), allow(dead_code))]
fn convert_bool(res: isize) -> Result<bool, Error> {
    match convert(res) {
        Ok(0) => Ok(false),
        Ok(1) => Ok(true),
        Ok(_) => unreachable!(),
        Err(e) => Err(e),
    }
}

#[cfg_attr(not(feature = "api-crypto-ec"), allow(dead_code))]
fn convert_unit(res: isize) -> Result<(), Error> {
    match convert(res) {
        Ok(0) => Ok(()),
        Ok(_) => unreachable!(),
        Err(e) => Err(e),
    }
}

fn convert_never(res: isize) -> Result<!, Error> {
    match convert(res) {
        Ok(_) => unreachable!(),
        Err(e) => Err(e),
    }
}
