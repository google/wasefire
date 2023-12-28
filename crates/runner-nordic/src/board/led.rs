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
use wasefire_board_api::led::Api;
use wasefire_board_api::{Error, Id, Support};

use crate::with_state;

pub enum Impl {}

impl Support<usize> for Impl {
    const SUPPORT: usize = 4;
}

impl Api for Impl {
    fn get(id: Id<Self>) -> Result<bool, Error> {
        with_state(|state| {
            let led = &mut state.leds[*id];
            led.is_set_low().map_err(|_| Error::world(0))
        })
    }

    fn set(id: Id<Self>, on: bool) -> Result<(), Error> {
        with_state(|state| {
            let led = &mut state.leds[*id];
            match on {
                false => led.set_high(),
                true => led.set_low(),
            }
            .map_err(|_| Error::world(0))
        })
    }
}
