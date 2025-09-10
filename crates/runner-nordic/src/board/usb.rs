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
#[cfg(feature = "_usb")]
use wasefire_board_api::usb::Api;
#[cfg(feature = "usb-ctap")]
use wasefire_board_api::usb::ctap::{Ctap, HasHid, WithHid};
#[cfg(feature = "usb-serial")]
use wasefire_board_api::usb::serial::{HasSerial, Serial, WithSerial};
use wasefire_error::{Code, Error};
use wasefire_protocol_usb::{HasRpc, Rpc};

use crate::with_state;

pub type Usb = Usbd<UsbPeripheral<'static>>;

pub enum Impl {}

pub type ProtocolImpl = wasefire_protocol_usb::Impl<'static, Usb, crate::board::usb::Impl>;

#[cfg(feature = "_usb")]
impl Api for Impl {
    #[cfg(feature = "usb-ctap")]
    type Ctap = WithHid<Impl>;
    #[cfg(feature = "usb-serial")]
    type Serial = WithSerial<Impl>;
}

impl HasRpc<'static, Usb> for Impl {
    fn with_rpc<R>(f: impl FnOnce(&mut Rpc<'static, Usb>) -> R) -> R {
        with_state(|state| f(&mut state.protocol))
    }

    #[cfg(not(feature = "test-vendor"))]
    fn vendor(_: &[u8]) -> Result<Box<[u8]>, Error> {
        Err(Error::user(Code::NotImplemented))
    }

    #[cfg(feature = "test-vendor")]
    fn vendor(request: &[u8]) -> Result<Box<[u8]>, Error> {
        use alloc::string::String;
        use core::fmt::Write;

        #[cfg(not(feature = "fpc2534"))]
        use data_encoding as _;

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
            return Ok(response);
        }
        if request == b"info\n" {
            let mut response = String::new();
            let running = header::running_side().unwrap();
            for side in wasefire_common::platform::Side::LIST {
                let header = header::Header::new(side);
                let timestamp = header.timestamp();
                let running = if side == running { "*" } else { " " };
                write!(&mut response, "{running} {side} {timestamp:08x} ").unwrap();
                for i in 0 .. 3 {
                    write!(&mut response, "{}", header.attempt(i).free() as u8).unwrap();
                }
                writeln!(&mut response).unwrap();
            }
            return Ok(response.into_bytes().into_boxed_slice());
        }
        if request == b"peak\n" {
            let peak = crate::allocator::peak();
            return Ok(alloc::format!("{peak}\n").into_bytes().into_boxed_slice());
        }
        if let Some(range) = request.strip_prefix(b"dump ") {
            let range: Option<_> = try {
                let range = core::str::from_utf8(range).ok()?.trim_end();
                let (start, limit) = range.split_once("-")?;
                let start = u32::from_str_radix(start, 16).ok()?;
                let limit = u32::from_str_radix(limit, 16).ok()?;
                (start, limit.checked_sub(start)?)
            };
            let Some((ptr, len)) = range else { return Err(Error::user(Code::InvalidArgument)) };
            let slice = unsafe { core::slice::from_raw_parts(ptr as *const u8, len as usize) };
            return Ok(slice.to_vec().into_boxed_slice());
        }
        #[cfg(feature = "fpc2534")]
        if let Some(frame) = request.strip_prefix(b"fpc ") {
            let mut hex = data_encoding::HEXLOWER_PERMISSIVE.specification();
            hex.ignore.push_str(" \n");
            let hex = hex.encoding().unwrap();
            let Ok(frame) = hex.decode(frame) else {
                return Err(Error::user(Code::InvalidArgument));
            };
            crate::board::fpc2534::vendor(frame);
            return Ok(Box::new([]));
        }
        Err(Error::user(Code::InvalidArgument))
    }
}

#[cfg(feature = "usb-ctap")]
impl HasHid for Impl {
    type UsbBus = Usb;

    fn with_hid<R>(f: impl FnOnce(&mut Ctap<Self::UsbBus>) -> R) -> R {
        with_state(|state| f(&mut state.ctap))
    }
}

#[cfg(feature = "usb-serial")]
impl HasSerial for Impl {
    type UsbBus = Usb;

    fn with_serial<R>(f: impl FnOnce(&mut Serial<Self::UsbBus>) -> R) -> R {
        with_state(|state| f(&mut state.serial))
    }
}
