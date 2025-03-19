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

use nrf52840_hal::gpio::{Input, Pin, PullUp};
use nrf52840_hal::gpiote::{Gpiote, GpioteChannel};
use wasefire_board_api::button::Api;
use wasefire_board_api::{Error, Id, Support};

use crate::with_state;

pub enum Impl {}

impl Support<usize> for Impl {
    #[cfg(feature = "board-devkit")]
    const SUPPORT: usize = 4;
    #[cfg(feature = "board-dongle")]
    const SUPPORT: usize = 1;
}

impl Api for Impl {
    fn enable(id: Id<Self>) -> Result<(), Error> {
        with_state(|state| {
            let pin = &state.buttons[*id].pin;
            channel(&state.gpiote, id).input_pin(pin).toggle().enable_interrupt();
            Ok(())
        })
    }

    fn disable(id: Id<Self>) -> Result<(), Error> {
        with_state(|state| {
            let pin = &state.buttons[*id].pin;
            channel(&state.gpiote, id).input_pin(pin).disable_interrupt();
            Ok(())
        })
    }
}

// We map channels and buttons one to one.
pub struct Button {
    pub pin: Pin<Input<PullUp>>,
}

impl Button {
    pub fn new(pin: Pin<Input<PullUp>>) -> Self {
        Button { pin }
    }
}

pub fn channel(gpiote: &Gpiote, i: Id<Impl>) -> GpioteChannel {
    match *i {
        0 => gpiote.channel0(),
        1 => gpiote.channel1(),
        2 => gpiote.channel2(),
        3 => gpiote.channel3(),
        4 => gpiote.channel4(),
        5 => gpiote.channel5(),
        6 => gpiote.channel6(),
        7 => gpiote.channel7(),
        _ => unreachable!(),
    }
}
