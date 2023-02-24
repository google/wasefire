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

use wasefire_applet_api::debug as api;

/// Prints a line to the debug output.
pub fn println(msg: &str) {
    if ENABLED {
        let params = api::println::Params { ptr: msg.as_ptr(), len: msg.len() };
        unsafe { api::println(params) };
    }
}

/// Whether debugging is enabled.
// We use an environment variable to avoid asking applets to forward features.
pub const ENABLED: bool = option_env!("FIRWASM_DEBUG").is_some();

/// Prints a line to the debug output.
///
/// This is similar to the standard `std::println!()` macro and supports the same formatting.
#[macro_export]
macro_rules! println {
    ($($args:tt)*) => {
        if $crate::debug::ENABLED {
            $crate::debug::println(&alloc::format!($($args)*));
        }
    };
}
