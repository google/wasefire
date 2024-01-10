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

//! Prints the debug time every second.

#![no_std]
wasefire::applet!();

use core::time::Duration;

fn main() {
    let timer = timer::Timer::new(|| debug!("{} micro-seconds", debug::time().unwrap()));
    timer.start(timer::Periodic, Duration::from_secs(1));
    timer.leak();
}
