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
mod crypto;
mod debug;
mod led;
mod rng;
pub mod timer;
#[cfg(feature = "usb")]
pub mod usb;

use std::sync::{Arc, Mutex};

use tokio::sync::mpsc::{Receiver, Sender};
use wasefire_board_api::{Api, Event};
use wasefire_store::FileStorage;

use self::timer::Timers;

pub struct State {
    pub sender: Sender<Event>,
    pub button: bool, // whether interrupts are enabled
    pub led: bool,
    pub timers: Timers,
    #[cfg(feature = "usb")]
    pub usb: usb::Usb,
    pub storage: Option<FileStorage>,
}

pub struct Board {
    pub receiver: Receiver<Event>,
    pub state: Arc<Mutex<State>>,
}

impl Api for Board {
    fn try_event(&mut self) -> Option<Event> {
        self.receiver.try_recv().ok()
    }

    fn wait_event(&mut self) -> Event {
        self.receiver.blocking_recv().unwrap()
    }

    type Storage = FileStorage;
    fn take_storage(&mut self) -> Option<Self::Storage> {
        self.state.lock().unwrap().storage.take()
    }

    type Button<'a> = &'a mut Self;
    fn button(&mut self) -> Self::Button<'_> {
        self
    }

    type Crypto<'a> = &'a mut Self;
    fn crypto(&mut self) -> Self::Crypto<'_> {
        self
    }

    type Debug<'a> = &'a mut Self;
    fn debug(&mut self) -> Self::Debug<'_> {
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

    #[cfg(feature = "usb")]
    type Usb<'a> = &'a mut Self;
    #[cfg(feature = "usb")]
    fn usb(&mut self) -> Self::Usb<'_> {
        self
    }
    #[cfg(not(feature = "usb"))]
    type Usb<'a> = wasefire_board_api::Unsupported;
    #[cfg(not(feature = "usb"))]
    fn usb(&mut self) -> Self::Usb<'_> {
        wasefire_board_api::Unsupported
    }
}
