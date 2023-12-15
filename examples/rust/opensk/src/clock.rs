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
// limitations under the License.use opensk_lib::api::clock::Clock;

use alloc::rc::Rc;
use core::cell::Cell;

use opensk_lib::api::clock::Clock;
use wasefire::clock::Mode::Oneshot;
use wasefire::clock::{Handler, Timer};

#[derive(Default)]
struct ClockHandler {
    triggered: Rc<Cell<bool>>,
}

impl Handler for ClockHandler {
    fn event(&self) {
        self.triggered.set(true);
    }
}

pub struct WasefireTimer {
    timer: Timer<ClockHandler>,
    elapsed: Rc<Cell<bool>>,
}

impl Default for WasefireTimer {
    fn default() -> Self {
        let elapsed = Rc::new(Cell::new(true));
        let triggered = elapsed.clone();
        // This is a bit wasteful to allocate a timer that we don't need. This could be optimized later.
        let timer = Timer::new(ClockHandler { triggered });
        Self { timer, elapsed }
    }
}

#[derive(Default)]
pub struct WasefireClock;

impl Clock for WasefireClock {
    type Timer = WasefireTimer;

    fn make_timer(&mut self, milliseconds: usize) -> Self::Timer {
        let elapsed = Rc::new(Cell::new(false));
        let triggered = elapsed.clone();
        let timer = Timer::new(ClockHandler { triggered });
        timer.start_ms(Oneshot, milliseconds);
        WasefireTimer { timer, elapsed }
    }

    fn is_elapsed(&mut self, timer: &Self::Timer) -> bool {
        timer.elapsed.get()
    }
}
