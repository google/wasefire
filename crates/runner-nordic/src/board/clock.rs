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

use nrf52840_hal::Timer;
use nrf52840_hal::pac::TIMER1;
use nrf52840_hal::timer::Periodic;
use wasefire_board_api::Error;
use wasefire_board_api::clock::Api;

use crate::with_state;

pub enum Impl {}

impl Api for Impl {
    fn uptime_us() -> Result<u64, Error> {
        with_state(|state| Ok(state.clock.uptime_us()))
    }
}

pub struct Clock {
    timer: Timer<TIMER1, Periodic>,
    base: u64,
}

impl Clock {
    pub fn new(timer: TIMER1) -> Self {
        let mut timer = Timer::periodic(timer);
        timer.enable_interrupt();
        timer.start((PERIOD - 1) as u32);
        Clock { timer, base: 0 }
    }

    pub fn tick(&mut self) {
        self.timer.reset_event();
        self.base += PERIOD;
    }

    fn uptime_us(&self) -> u64 {
        let mut counter = self.timer.read() as u64;
        // We assume the timer can't wrap around twice without self.base being updated (this would
        // mean not processing interrupts for more than 1 hour).
        if self.timer.event_compare_cc0().read().events_compare().bit() {
            counter = self.timer.read() as u64 + PERIOD;
        }
        self.base + counter
    }
}

const PERIOD: u64 = u32::MAX as u64 + 1;
