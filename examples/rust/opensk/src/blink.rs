// Copyright 2025 Google LLC
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

use wasefire::{led, timer};

pub(crate) struct Blink {
    timer: timer::Timer<BlinkHandler>,
}

impl Blink {
    pub(crate) fn new_ms(period_ms: usize) -> Self {
        let timer = timer::Timer::new(BlinkHandler);
        led::set(0, led::On);
        timer.start_ms(timer::Mode::Periodic, period_ms);
        Blink { timer }
    }
}

impl Drop for Blink {
    fn drop(&mut self) {
        self.timer.stop();
        led::set(0, led::Off);
    }
}

struct BlinkHandler;

impl timer::Handler for BlinkHandler {
    fn event(&self) {
        led::set(0, !led::get(0));
    }
}
