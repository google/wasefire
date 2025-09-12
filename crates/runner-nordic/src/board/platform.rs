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

use header::Header;
use nrf52840_hal::pac::FICR;
use wasefire_board_api::Error;
use wasefire_board_api::platform::Api;
use wasefire_common::addr_of_symbol;
use wasefire_common::platform::Side;
use wasefire_error::Code;
use wasefire_sync::Once;

pub mod update;

pub enum Impl {}

impl Api for Impl {
    type Protocol = crate::board::usb::ProtocolImpl;

    type Update = update::Impl;

    fn serial() -> Cow<'static, [u8]> {
        serial().into()
    }

    fn running_side() -> Side {
        header::running_side().unwrap()
    }

    fn running_version() -> Cow<'static, [u8]> {
        let side = Self::running_side();
        let header = Header::new(side);
        header.timestamp().to_be_bytes().to_vec().into()
    }

    fn opposite_version() -> Result<Cow<'static, [u8]>, Error> {
        if addr_of_symbol!(__sother) == addr_of_symbol!(__eother) {
            return Err(Error::world(Code::NotEnough));
        }
        let side = Self::running_side().opposite();
        let header = Header::new(side);
        if header.has_firmware() {
            Ok(header.timestamp().to_be_bytes().to_vec().into())
        } else {
            Err(Error::world(Code::NotFound))
        }
    }

    fn reboot() -> Result<!, Error> {
        reboot()
    }
}

pub fn reboot() -> ! {
    wasefire_logger::flush();
    nrf52840_hal::pac::SCB::sys_reset()
}

pub fn init_serial(ficr: &FICR) {
    let low = ficr.deviceid[0].read().deviceid().bits();
    let high = ficr.deviceid[1].read().deviceid().bits();
    let mut serial = [0; 8];
    serial[.. 4].copy_from_slice(&high.to_be_bytes());
    serial[4 ..].copy_from_slice(&low.to_be_bytes());
    SERIAL.call_once(|| serial);
}

pub fn serial() -> &'static [u8; 8] {
    SERIAL.get().unwrap()
}

static SERIAL: Once<[u8; 8]> = Once::new();
