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

use core::ops::DerefMut;

use nrf52840_hal::gpio::{Input, Pin, PullUp};
use nrf52840_hal::gpiote::{Gpiote, GpioteChannel};
use wasefire_board_api as board;

impl board::button::Api for &mut crate::tasks::Board {
    fn count(&mut self) -> usize {
        critical_section::with(|cs| self.0.borrow_ref_mut(cs).buttons.len())
    }

    fn enable(&mut self, i: usize) -> Result<(), board::Error> {
        critical_section::with(|cs| try {
            let mut state = self.0.borrow_ref_mut(cs);
            let state = state.deref_mut();
            let button = state.buttons.get_mut(i).ok_or(board::Error::User)?;
            channel(&state.gpiote, i).input_pin(&button.pin).toggle().enable_interrupt();
        })
    }

    fn disable(&mut self, i: usize) -> Result<(), board::Error> {
        critical_section::with(|cs| try {
            let mut state = self.0.borrow_ref_mut(cs);
            let state = state.deref_mut();
            let button = state.buttons.get_mut(i).ok_or(board::Error::User)?;
            channel(&state.gpiote, i).input_pin(&button.pin).disable_interrupt();
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

pub fn channel(gpiote: &Gpiote, i: usize) -> GpioteChannel {
    match i {
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
