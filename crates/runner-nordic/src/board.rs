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

use wasefire_board_api::{self as board, AppletMemory, Event, Failure, Singleton};
use wasefire_scheduler as scheduler;

use crate::{Board, with_state};

pub mod applet;
pub mod button;
mod clock;
#[cfg(feature = "_crypto")]
mod crypto;
mod debug;
#[cfg(feature = "fpc2534")]
pub mod fpc2534;
#[cfg(feature = "gpio")]
pub mod gpio;
pub mod led;
pub mod platform;
#[cfg(feature = "radio-ble")]
pub mod radio;
mod rng;
pub mod timer;
#[cfg(feature = "uart")]
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

    fn vendor(mem: impl AppletMemory, x1: u32, x2: u32, x3: u32, x4: u32) -> Result<u32, Failure> {
        #[cfg(not(feature = "test-vendor"))]
        let _ = &mem;
        match (x1, x2, x3, x4) {
            #[cfg(feature = "test-vendor")]
            (0, _, _, _) => syscall_test::process(mem, x2, x3, x4),
            #[cfg(feature = "gpio")]
            (0x80000000, x, y, z) => Ok(gpio::syscall(x, y, z)?),
            _ => Err(Failure::TRAP),
        }
    }

    type Applet = applet::Impl;
    type Button = button::Impl;
    type Clock = clock::Impl;
    #[cfg(feature = "_crypto")]
    type Crypto = crypto::Impl;
    type Debug = debug::Impl;
    #[cfg(feature = "fpc2534")]
    type Fingerprint = fpc2534::Impl;
    #[cfg(feature = "gpio")]
    type Gpio = gpio::Impl;
    type Led = led::Impl;
    type Platform = platform::Impl;
    #[cfg(feature = "radio-ble")]
    type Radio = radio::Impl;
    type Rng = rng::Impl;
    type Storage = crate::storage::Storage;
    type Timer = timer::Impl;
    #[cfg(feature = "uart")]
    type Uart = uart::Impl;
    #[cfg(feature = "_usb")]
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
