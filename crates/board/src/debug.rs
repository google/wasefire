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

//! Debugging and testing interface.

use wasefire_logger as log;

use crate::Unsupported;

/// Debugging and testing interface.
pub trait Api {
    /// Maximum value returned by [`Self::time()`] before wrapping.
    const MAX_TIME: u64;

    /// Prints a line with timestamp.
    fn println(line: &str) {
        let time = Self::time();
        log::println!("{}.{:06}: {}", time / 1000000, time % 1000000, line);
    }

    /// Returns the time in micro-seconds since some initial event.
    ///
    /// This wraps once [`Self::MAX_TIME`] is reached. In particular, a maximum value of zero
    /// equivalent to not supporting this API.
    fn time() -> u64;

    /// Exits the platform with a success/failure result.
    fn exit(success: bool) -> !;
}

impl Api for Unsupported {
    const MAX_TIME: u64 = 0;

    fn time() -> u64 {
        0
    }

    fn exit(success: bool) -> ! {
        log::panic!("exit({}) called", success)
    }
}
