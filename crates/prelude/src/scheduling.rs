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

//! Provides API for scheduling.

use wasefire_applet_api::scheduling as api;

/// Waits until a callback is called.
///
/// This is similar to how `wfi` (wait for interrupt) and `wfe` (wait for event) work. When calling
/// this function, the applet stops executing. It resumes execution when a callback is scheduled.
/// This function returns once the callback executed. It is possible for concurrence reasons that
/// multiple callbacks execute. Callbacks have priority over resuming execution of a code waiting
/// for callbacks.
pub fn wait_for_callback() {
    unsafe { api::wait_for_callback() };
}

/// Returns how many callbacks are pending.
pub fn num_pending_callbacks() -> usize {
    let api::num_pending_callbacks::Results { count } = unsafe { api::num_pending_callbacks() };
    count
}

/// Waits until a condition is satisfied.
pub fn wait_until(mut cond: impl FnMut() -> bool) {
    while !cond() {
        wait_for_callback();
    }
}

/// Waits for callbacks indefinitely.
pub fn wait_indefinitely() -> ! {
    loop {
        wait_for_callback();
    }
}
