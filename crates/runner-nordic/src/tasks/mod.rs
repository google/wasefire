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

use wasefire_board_api::{self as board, Event};
use wasefire_scheduler as scheduler;

use crate::Board;

pub mod button;
pub mod clock;
mod crypto;
mod led;
mod rng;
pub mod usb;

impl core::fmt::Debug for Board {
    fn fmt(&self, _: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Ok(())
    }
}

impl board::Api for Board {
    fn try_event(&mut self) -> Option<board::Event> {
        critical_section::with(|cs| self.0.borrow_ref_mut(cs).events.pop())
    }

    fn wait_event(&mut self) -> board::Event {
        loop {
            match self.try_event() {
                None => Events::wait(),
                Some(event) => break event,
            }
        }
    }

    fn breakpoint(&mut self) {
        cortex_m::asm::bkpt();
    }

    type Storage = crate::storage::Storage;
    fn take_storage(&mut self) -> Option<Self::Storage> {
        critical_section::with(|cs| self.0.borrow_ref_mut(cs).storage.take())
    }

    type Button<'a> = &'a mut Self;
    fn button(&mut self) -> Self::Button<'_> {
        self
    }

    type Crypto<'a> = &'a mut Self;
    fn crypto(&mut self) -> Self::Crypto<'_> {
        self
    }

    type Led<'a> = &'a mut Self;
    fn led(&mut self) -> Self::Led<'_> {
        self
    }

    type Rng<'a> = &'a mut Self;
    fn rng(&mut self) -> Self::Rng<'_> {
        self
    }

    type Timer<'a> = &'a mut Self;
    fn timer(&mut self) -> Self::Timer<'_> {
        self
    }

    type Usb<'a> = &'a mut Self;
    fn usb(&mut self) -> Self::Usb<'_> {
        self
    }
}

#[derive(Default)]
pub struct Events(scheduler::Events);

impl Events {
    pub fn push(&mut self, event: Event) {
        self.0.push(event);
        cortex_m::asm::sev();
    }

    fn pop(&mut self) -> Option<Event> {
        self.0.pop()
    }

    // May return even if there are no events.
    fn wait() {
        cortex_m::asm::wfe();
    }
}
