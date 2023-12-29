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

pub mod button;
mod debug;
mod led;
mod rng;
mod storage;
pub mod timer;
pub mod uart;
#[cfg(feature = "usb")]
pub mod usb;

use tokio::sync::mpsc::Sender;
use wasefire_board_api::{Api, Event, Unsupported};
use wasefire_store::FileStorage;

use crate::RECEIVER;

pub struct State {
    pub sender: Sender<Event<Board>>,
    pub button: bool, // whether interrupts are enabled
    pub led: bool,
    pub timers: timer::Timers,
    pub uarts: uart::Uarts,
    #[cfg(feature = "usb")]
    pub usb: usb::Usb,
    pub storage: Option<FileStorage>,
    #[cfg(feature = "web")]
    pub web: web_server::Client,
}

pub enum Board {}

impl Api for Board {
    fn try_event() -> Option<Event<Board>> {
        RECEIVER.lock().unwrap().as_mut().unwrap().try_recv().ok()
    }

    fn wait_event() -> Event<Board> {
        RECEIVER.lock().unwrap().as_mut().unwrap().blocking_recv().unwrap()
    }

    fn syscall(x1: u32, x2: u32, x3: u32, x4: u32) -> Option<u32> {
        match (x1, x2, x3, x4) {
            // The syscall_test example relies on this.
            (0, 0, 0, x) => Some(x),
            _ => None,
        }
    }

    type Button = button::Impl;
    type Crypto = Unsupported;
    type Debug = debug::Impl;
    type Gpio = Unsupported;
    type Led = led::Impl;
    type Platform = Unsupported;
    type Radio = Unsupported;
    type Rng = rng::Impl;
    type Storage = storage::Impl;
    type Timer = timer::Impl;
    type Uart = uart::Impl;
    #[cfg(feature = "usb")]
    type Usb = usb::Impl;
    #[cfg(not(feature = "usb"))]
    type Usb = Unsupported;
}
