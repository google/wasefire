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

use nrf52840_hal::prelude::{OutputPin, StatefulOutputPin};

impl board::led::Api for crate::tasks::Board {
    fn count(&mut self) -> usize {
        critical_section::with(|cs| self.0.borrow_ref(cs).leds.len())
    }

    fn get(&mut self, i: usize) -> Result<bool, board::Error> {
        critical_section::with(|cs| {
            let leds = &mut self.0.borrow_ref_mut(cs).leds;
            let led = leds.get_mut(i).ok_or(board::Error::User)?;
            led.is_set_low().map_err(|_| board::Error::World)
        })
    }

    fn set(&mut self, i: usize, on: bool) -> Result<(), board::Error> {
        critical_section::with(|cs| {
            let leds = &mut self.0.borrow_ref_mut(cs).leds;
            let led = leds.get_mut(i).ok_or(board::Error::User)?;
            match on {
                false => led.set_high(),
                true => led.set_low(),
            }
            .map_err(|_| board::Error::World)
        })
    }
}
