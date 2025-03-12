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

use core::cell::RefCell;

use critical_section::Mutex;
use wasefire_board_api::{self as board, Event};
use wasefire_error::Error;
use wasefire_scheduler as scheduler;

mod applet;
mod crypto;
mod debug;
mod platform;
mod storage;
pub mod timer;
pub mod usb;

struct State {
    events: Events,
    applet: applet::State,
    platform: platform::State,
    storage: storage::State,
    timer: timer::State,
    usb: usb::State,
}

static STATE: Mutex<RefCell<Option<State>>> = Mutex::new(RefCell::new(None));

fn with_state<R>(f: impl FnOnce(&mut State) -> R) -> R {
    critical_section::with(|cs| f(STATE.borrow_ref_mut(cs).as_mut().unwrap()))
}

pub fn init() {
    let state = State {
        events: Events::default(),
        applet: applet::init(),
        platform: platform::init(),
        storage: storage::init(),
        timer: timer::init(),
        usb: usb::init(),
    };
    critical_section::with(|cs| STATE.replace(cs, Some(state)));
}

pub enum Board {}

impl board::Api for Board {
    fn try_event() -> Option<Event<Self>> {
        with_state(|state| state.events.pop())
    }

    fn wait_event() -> Event<Self> {
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
            #[cfg(feature = "test-vendor")]
            (0, 0, 0, x) => Some(Error::decode(x as i32)),
            _ => None,
        }
    }

    type Applet = applet::Impl;
    type Crypto = crypto::Impl;
    type Debug = debug::Impl;
    type Platform = platform::Impl;
    type Storage = storage::Impl;
    type Timer = timer::Impl;
}

#[derive(Default)]
pub struct Events(scheduler::Events<Board>);

impl Events {
    pub fn push(&mut self, event: Event<Board>) {
        self.0.push(event);
    }

    fn pop(&mut self) -> Option<Event<Board>> {
        self.0.pop()
    }

    // May return even if there are no events.
    fn wait() {
        with_state(|state| {
            if state.events.0.is_empty() {
                // Interrupts don't need to be enabled for WFI to wake the CPU. This permits being
                // sure that we don't sleep if an interrupt triggered after the test but before WFI,
                // because WFI will see the pending interrupt.
                riscv::asm::wfi();
            }
        });
    }
}
