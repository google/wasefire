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

//! Prints the debug time for each component every second. Alternate every second between being idle
//! and busy.

#![no_std]
wasefire::applet!();

use core::time::Duration;

fn main() {
    let prev = sync::Mutex::new(debug::perf());
    let idle = sync::AtomicBool::new(true);
    let timer = timer::Timer::new(move || {
        let next = debug::perf();
        debug!("{:?}", next - core::mem::replace(&mut prev.lock(), next));
        if idle.fetch_xor(true, sync::Ordering::SeqCst) {
            while scheduling::num_pending_callbacks() == 0 {}
        }
    });
    timer.start(timer::Periodic, Duration::from_secs(1));
    timer.leak();
}
