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

//! Provides debugging facilities.

// We don't try to optimize debugging. The user should debug with a host runtime where the applet
// size (and other performance) doesn't matter. This permits to have a simple debugging that is
// completely excluded from release applets.

use bytemuck::Zeroable;
use wasefire_applet_api::debug as api;
pub use wasefire_applet_api::debug::Perf;
use wasefire_error::Error;

use crate::convert;

/// Prints a line to the debug output.
pub fn println(msg: &str) {
    if ENABLED {
        let params = api::println::Params { ptr: msg.as_ptr(), len: msg.len() };
        unsafe { api::println(params) };
    }
}

/// Returns the time in micro-seconds since some initial event.
///
/// The time may wrap before using all 64 bits.
pub fn time() -> Result<u64, Error> {
    let mut result: u64 = 0;
    let params = api::time::Params { ptr: &mut result as *mut _ };
    let api::time::Results { res } = unsafe { api::time(params) };
    convert(res).map(|_| result)
}

/// Returns the time in micro-seconds since some initial event, split by component.
pub fn perf() -> Perf {
    let mut result = Perf::zeroed();
    let params = api::perf::Params { ptr: &mut result as *mut Perf as *mut u8 };
    unsafe { api::perf(params) };
    result
}

/// Whether debugging is enabled.
// We use an environment variable to avoid asking applets to forward features.
pub const ENABLED: bool = option_env!("WASEFIRE_DEBUG").is_some();

#[deprecated(since = "0.1.2", note = "use debug! instead")]
#[macro_export]
macro_rules! println {
    ($($args:tt)*) => {
        if $crate::debug::ENABLED {
            $crate::debug::println(&alloc::format!($($args)*));
        }
    };
}

/// Prints a line to the debug output.
///
/// This is similar to the standard `std::println!()` macro and supports the same formatting.
#[macro_export]
macro_rules! debug {
    ($($args:tt)*) => {
        if $crate::debug::ENABLED {
            $crate::debug::println(&alloc::format!($($args)*));
        }
    };
}

/// Exits the platform indicating success of failure.
pub fn exit(success: bool) -> ! {
    let params = api::exit::Params { code: if success { 0 } else { 1 } };
    unsafe { api::exit(params) };
    unreachable!()
}

/// Asserts that a condition holds and exits with an error otherwise.
#[track_caller]
pub fn assert(condition: bool) {
    if !condition {
        debug!("Assertion failed at {}", core::panic::Location::caller());
        exit(false);
    }
}

/// Asserts that an equality holds and exits with an error otherwise.
#[track_caller]
pub fn assert_eq<T: ?Sized + Eq + core::fmt::Debug>(x: &T, y: &T) {
    if x != y {
        debug!("{x:?} != {y:?} at {}", core::panic::Location::caller());
        exit(false);
    }
}
