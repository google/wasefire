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
mod led;
mod rng;
pub mod timer;
mod usb;

use std::sync::{Arc, Mutex};

use tokio::sync::mpsc::{Receiver, Sender};
use usb_device::prelude::UsbDevice;
use usbip_device::UsbIpBus;
use wasefire_board_api::usb::serial::Serial;
use wasefire_board_api::{Api, Event};
use wasefire_store::FileStorage;

use self::timer::Timers;

pub struct State {
    pub sender: Sender<Event>,
    pub button: bool, // whether interrupts are enabled
    pub led: bool,
    pub timers: Timers,
    pub serial: Serial<'static, UsbIpBus>,
    pub usb_dev: UsbDevice<'static, UsbIpBus>,
    pub storage: Option<FileStorage>,
}

pub struct Board {
    pub receiver: Receiver<Event>,
    pub state: Arc<Mutex<State>>,
}

impl Api for Board {
    type Storage = FileStorage;

    fn try_event(&mut self) -> Option<Event> {
        self.receiver.try_recv().ok()
    }

    fn wait_event(&mut self) -> Event {
        self.receiver.blocking_recv().unwrap()
    }

    fn take_storage(&mut self) -> Option<Self::Storage> {
        self.state.lock().unwrap().storage.take()
    }
}

impl State {
    pub fn poll(&mut self) -> bool {
        self.usb_dev.poll(&mut [self.serial.port()])
    }
}
