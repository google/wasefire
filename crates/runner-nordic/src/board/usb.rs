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

use alloc::boxed::Box;

use nrf52840_hal::usbd::{UsbPeripheral, Usbd};
use wasefire_board_api::usb::serial::{HasSerial, Serial, WithSerial};
use wasefire_board_api::usb::Api;
use wasefire_error::{Code, Error};
use wasefire_protocol_usb::{HasRpc, Rpc};

use crate::with_state;

pub type Usb = Usbd<UsbPeripheral<'static>>;

pub enum Impl {}

pub type ProtocolImpl = wasefire_protocol_usb::Impl<'static, Usb, crate::board::usb::Impl>;

impl Api for Impl {
    type Serial = WithSerial<Impl>;
}

impl HasRpc<'static, Usb> for Impl {
    fn with_rpc<R>(f: impl FnOnce(&mut Rpc<'static, Usb>) -> R) -> R {
        with_state(|state| f(&mut state.protocol))
    }

    fn vendor(request: &[u8]) -> Result<Box<[u8]>, Error> {
        if let Some(request) = request.strip_prefix(b"echo ") {
            let mut response = request.to_vec().into_boxed_slice();
            for x in &mut response {
                if x.is_ascii_alphabetic() {
                    *x ^= 0x20;
                }
                if matches!(*x, b'I' | b'O' | b'i' | b'o') {
                    *x ^= 0x6;
                }
            }
            Ok(response)
        } else {
            Err(Error::user(Code::InvalidArgument))
        }
    }
}

impl HasSerial for Impl {
    type UsbBus = Usb;

    fn with_serial<R>(f: impl FnOnce(&mut Serial<Self::UsbBus>) -> R) -> R {
        with_state(|state| f(&mut state.serial))
    }
}
