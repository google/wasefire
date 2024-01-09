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

use wasefire_board_api::{self as board, Event, Singleton};
use wasefire_error::Error;
use wasefire_scheduler as scheduler;

use crate::{with_state, Board};

pub mod button;
pub mod clock;
mod crypto;
mod debug;
pub mod gpio;
pub mod led;
pub mod platform;
pub mod radio;
mod rng;
pub mod uart;
pub mod usb;

impl board::Api for Board {
    fn try_event() -> Option<board::Event<Board>> {
        with_state(|state| state.events.pop())
    }

    fn wait_event() -> board::Event<Board> {
        loop {
            match Self::try_event() {
                None => Events::wait(),
                Some(event) => break event,
            }
        }
    }

    fn syscall(x1: u32, x2: u32, x3: u32, x4: u32) -> Option<Result<u32, Error>> {
        match (x1, x2, x3, x4) {
            // The syscall_test example relies on this.
            (0, 0, 0, x) => Some(Error::decode(x as i32)),
            _ => None,
        }
    }

    type Button = button::Impl;
    type Crypto = crypto::Impl;
    type Debug = debug::Impl;
    type Gpio = gpio::Impl;
    type Led = led::Impl;
    type Platform = platform::Impl;
    type Radio = radio::Impl;
    type Rng = rng::Impl;
    type Storage = crate::storage::Storage;
    type Timer = clock::Impl;
    type Uart = uart::Impl;
    type Usb = usb::Impl;
}

impl Singleton for crate::storage::Storage {
    fn take() -> Option<Self> {
        with_state(|state| state.storage.take())
    }
}

#[derive(Default)]
pub struct Events(scheduler::Events<Board>);

impl Events {
    pub fn push(&mut self, event: Event<Board>) {
        self.0.push(event);
        cortex_m::asm::sev();
    }

    fn pop(&mut self) -> Option<Event<Board>> {
        self.0.pop()
    }

    // May return even if there are no events.
    fn wait() {
        cortex_m::asm::wfe();
    }
}
