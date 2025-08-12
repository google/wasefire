// Copyright 2025 Google LLC
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

use alloc::vec::Vec;

use wasefire_board_api::applet::{Handlers, Memory};
use wasefire_board_api::vendor::Api;
#[cfg(feature = "test-vendor")]
use wasefire_board_api::vendor::Event as BoardEvent;
use wasefire_board_api::{Error, Failure};

#[cfg(feature = "test-vendor")]
use crate::with_state;

#[cfg(feature = "ble-adv")]
pub mod ble_adv;

pub enum Impl {}

impl Api for Impl {
    type Event = Event;
    type Key = Key;

    fn key(event: &Self::Event) -> Self::Key {
        match *event {
            #[cfg(feature = "ble-adv")]
            Event::BleAdv => Key::BleAdv,
            #[cfg(feature = "test-vendor")]
            Event::Vendor(ref event) => Key::Vendor(syscall_test::key(event)),
        }
    }

    fn syscall(
        memory: impl Memory, handlers: impl Handlers<Self::Key>, x1: u32, x2: u32, x3: u32, x4: u32,
    ) -> Result<u32, Failure> {
        let _ = (&memory, &handlers);
        match (x1, x2, x3, x4) {
            #[cfg(feature = "test-vendor")]
            (0, _, _, _) => with_state(|state| {
                let push = |event| state.events.push(BoardEvent(Event::Vendor(event)).into());
                syscall_test::syscall(memory, handlers, push, x2, x3, x4)
            }),
            #[cfg(feature = "gpio")]
            (0x80000000, x, y, z) => Ok(crate::board::gpio::syscall(x, y, z)?),
            #[cfg(feature = "ble-adv")]
            (::ble_adv::SYSCALL_ID, x, y, z) => Ok(ble_adv::syscall(memory, handlers, x, y, z)?),
            _ => Err(Failure::TRAP),
        }
    }

    fn callback(
        memory: impl Memory, handlers: impl Handlers<Self::Key>, event: Self::Event,
        params: &mut Vec<u32>,
    ) {
        let _ = (&memory, &handlers, &params);
        match event {
            #[cfg(feature = "ble-adv")]
            Event::BleAdv => (),
            #[cfg(feature = "test-vendor")]
            Event::Vendor(event) => syscall_test::callback(memory, handlers, event, params),
        }
    }

    fn disable(key: Self::Key) -> Result<(), Error> {
        match key {
            #[cfg(feature = "ble-adv")]
            Key::BleAdv => ble_adv::disable(),
            #[cfg(feature = "test-vendor")]
            Key::Vendor(key) => syscall_test::disable(key),
        }
    }
}

#[cfg_attr(feature = "debug", derive(defmt::Format))]
#[derive(Debug, PartialEq, Eq)]
pub enum Event {
    #[cfg(feature = "ble-adv")]
    BleAdv,
    #[cfg(feature = "test-vendor")]
    Vendor(syscall_test::Event),
}

#[cfg_attr(feature = "debug", derive(defmt::Format))]
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Key {
    #[cfg(feature = "ble-adv")]
    BleAdv,
    #[cfg(feature = "test-vendor")]
    Vendor(syscall_test::Key),
}

#[cfg(feature = "test-vendor")]
impl From<syscall_test::Key> for Key {
    fn from(key: syscall_test::Key) -> Self {
        Key::Vendor(key)
    }
}
