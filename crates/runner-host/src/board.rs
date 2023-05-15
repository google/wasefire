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
#[cfg(feature = "usb")]
pub mod usb;

use tokio::sync::mpsc::Sender;
use wasefire_board_api::{Api, Event, Unsupported};
use wasefire_store::FileStorage;

use self::timer::Timers;
use crate::RECEIVER;

pub struct State {
    pub sender: Sender<Event<Board>>,
    pub button: bool, // whether interrupts are enabled
    pub led: bool,
    pub timers: Timers,
    #[cfg(feature = "usb")]
    pub usb: usb::Usb,
    pub storage: Option<FileStorage>,
}

pub enum Board {}

impl Api for Board {
    fn try_event() -> Option<Event<Board>> {
        RECEIVER.lock().unwrap().as_mut().unwrap().try_recv().ok()
    }

    fn wait_event() -> Event<Board> {
        RECEIVER.lock().unwrap().as_mut().unwrap().blocking_recv().unwrap()
    }

    type Button = button::Impl;
    type Crypto = Unsupported;
    type Debug = debug::Impl;
    type Led = led::Impl;
    type Rng = rng::Impl;
    type Storage = storage::Impl;
    type Timer = timer::Impl;
    #[cfg(feature = "usb")]
    type Usb = usb::Impl;
    #[cfg(not(feature = "usb"))]
    type Usb = Unsupported;
}
