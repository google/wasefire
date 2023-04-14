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

use core::sync::atomic::{AtomicU32, Ordering};

use cortex_m::peripheral::SYST;
use cortex_m_rt::exception;

const CLOCK_FREQ: u32 = 64_000_000;
const TICK_FREQ: u32 = 10_000;
const TICK_TO_MICRO: u32 = 1_000_000u32 / TICK_FREQ;

/// Number of ticks since boot.
static COUNT: AtomicU32 = AtomicU32::new(0);
defmt::timestamp!("{=u32:us}", TICK_TO_MICRO.wrapping_mul(COUNT.load(Ordering::SeqCst)));

pub fn init(mut syst: SYST) {
    // The minus one at the end is by definition of reload value. The added 0.5 is to avoid biasing
    // through truncation.
    let reload = (CLOCK_FREQ + TICK_FREQ / 2) / TICK_FREQ - 1;
    assert!((1 .. 0x10000).contains(&reload)); // SYST::set_reload() precondition
    syst.set_reload(reload);
    syst.clear_current();
    syst.enable_counter();
    syst.enable_interrupt();
}

#[exception]
fn SysTick() {
    COUNT.fetch_add(1, Ordering::SeqCst);
}
