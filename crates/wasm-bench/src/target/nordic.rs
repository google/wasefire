// Copyright 2024 Google LLC
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
use panic_rtt_target as _;

pub(crate) fn clock_ms() -> u64 {
    // The frequency is 64MHz and we want milli-seconds.
    (ticks() >> 6) / 1000
}

#[macro_export]
macro_rules! println {
    ($($x:tt)*) => { rtt_target::rprintln!($($x)*) };
}

pub(crate) fn init() {
    rtt_target::rtt_init_print!();
    crate::allocator::init();
    let c = nrf52840_hal::pac::CorePeripherals::take().unwrap();
    let mut syst = c.SYST;
    syst.set_reload(0xffffff);
    syst.clear_current();
    syst.enable_counter();
    syst.enable_interrupt();
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
    // The counter wraps every 2^24 ticks (262.144 milliseconds).
    (COUNT.load(Ordering::Relaxed) as u64) << 24
}

fn ticks_low() -> u64 {
    (0xffffff - SYST::get_current()) as u64
}

#[exception]
fn SysTick() {
    static USAGE: AtomicU32 = AtomicU32::new(0);
    COUNT.fetch_add(1, Ordering::Relaxed);
    if COUNT.load(Ordering::Relaxed) & 0xf == 0 {
        USAGE.fetch_max(crate::allocator::usage() as u32, Ordering::Relaxed);
        println!("Peak RAM usage: {}", USAGE.load(Ordering::Relaxed));
    }
}
