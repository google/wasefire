// Copyright 2023 Google LLC
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

use alloc::borrow::Cow;
use alloc::vec::Vec;

use header::{Header, Side};
use wasefire_board_api::platform::Api;
use wasefire_board_api::Error;

use crate::with_state;

pub mod update;

pub enum Impl {}

impl Api for Impl {
    type Protocol = crate::board::usb::ProtocolImpl;

    type Update = update::Impl;

    fn serial() -> Cow<'static, [u8]> {
        with_state(|state| {
            let low = state.ficr.deviceid[0].read().deviceid().bits();
            let high = state.ficr.deviceid[1].read().deviceid().bits();
            let mut serial = Vec::with_capacity(8);
            serial.extend_from_slice(&high.to_be_bytes());
            serial.extend_from_slice(&low.to_be_bytes());
            serial.into()
        })
    }

    fn version() -> Cow<'static, [u8]> {
        let side = Side::current().unwrap();
        let header = Header::new(side);
        header.timestamp().to_be_bytes().to_vec().into()
    }

    fn reboot() -> Result<!, Error> {
        reboot()
    }
}

pub fn reboot() -> ! {
    wasefire_logger::flush();
    nrf52840_hal::pac::SCB::sys_reset()
}
