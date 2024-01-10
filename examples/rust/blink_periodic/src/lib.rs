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

//! Demonstrates simple periodic timer usage.
//!
//! The applet additionally relies on LED. The applet behavior is similar to the blink example. The
//! first LED blinks with a period of 1 second and 50% duty cycle. However the implementation
//! differs, a periodic timer of 500ms is used to toggle the led.

#![no_std]
wasefire::applet!();

use core::cell::Cell;
use core::time::Duration;

fn main() {
    let n = led::count();
    assert!(n > 0, "No LEDs.");
    let i = Cell::new(0);
    let x = Cell::new(led::On);
    let step = move || {
        let i_ = i.get();
        let x_ = x.get();
        led::set(i_, x_);
        x.set(!x_);
        i.set((i_ + matches!(!x_, led::On) as usize) % n);
    };
    step();
    let timer = timer::Timer::new(step);
    timer.start(timer::Periodic, Duration::from_millis(500));
    scheduling::wait_indefinitely();
}
