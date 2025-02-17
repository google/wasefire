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

#[cfg(feature = "debug")]
defmt::timestamp!("{=u32:us}", uptime_us() as u32);

/// Returns the time in micro-seconds since [`init()`] was called.
///
/// This uses a 56 bits counter at 64MHz. So the granularity is actually better than micro-seconds.
/// And the wrapping time is almost 2285 years.
pub fn uptime_us() -> u64 {
    // The frequency is 64MHz and we want micro-seconds.
    ticks() >> 6
}

static COUNT: AtomicU32 = AtomicU32::new(0);

fn ticks() -> u64 {
    let old_low = ticks_low();
    let mut high = ticks_high();
    let low = ticks_low();
    if low < old_low {
        // Time wrapped between both low measures.
        high = ticks_high();
    }
    high | low
}

fn ticks_high() -> u64 {
    // The counter wraps every 2^24 ticks (about 16.77 seconds).
    (COUNT.load(Ordering::SeqCst) as u64) << 24
}

fn ticks_low() -> u64 {
    (0xffffff - SYST::get_current()) as u64
}

pub fn init(mut syst: SYST) {
    syst.set_reload(0xffffff);
    syst.clear_current();
    syst.enable_counter();
    syst.enable_interrupt();
}

#[exception]
fn SysTick() {
    COUNT.fetch_add(1, Ordering::SeqCst);
}
