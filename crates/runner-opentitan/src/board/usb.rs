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

use alloc::boxed::Box;

use usb_device::bus::UsbBusAllocator;
use usb_device::class::UsbClass;
use usb_device::device::UsbDevice;
#[cfg(feature = "usb-ctap")]
use usbd_hid::hid_class::HIDClass;
#[cfg(feature = "_usb")]
use wasefire_board_api::usb::Api;
#[cfg(feature = "usb-ctap")]
use wasefire_board_api::usb::ctap::{Ctap, HasHid, WithHid};
use wasefire_error::{Code, Error};
use wasefire_protocol_usb::{HasRpc, Rpc};

use crate::board::with_state;
use crate::usb::Usb;

pub struct State {
    protocol: wasefire_protocol_usb::Rpc<'static, Usb>,
    #[cfg(feature = "usb-ctap")]
    ctap: Ctap<'static, Usb>,
    device: UsbDevice<'static, Usb>,
}

pub fn init() -> State {
    let usb_bus = Box::leak(Box::new(UsbBusAllocator::new(Usb::new())));
    let protocol = wasefire_protocol_usb::Rpc::new(usb_bus);
    #[cfg(feature = "usb-ctap")]
    const CTAP_REPORT_DESCRIPTOR: &[u8] = &[
        0x06, 0xd0, 0xf1, 0x09, 0x01, 0xa1, 0x01, 0x09, 0x20, 0x15, 0x00, 0x26, 0xff, 0x00, 0x75,
        0x08, 0x95, 0x40, 0x81, 0x02, 0x09, 0x21, 0x15, 0x00, 0x26, 0xff, 0x00, 0x75, 0x08, 0x95,
        0x40, 0x91, 0x02, 0xc0,
    ];
    #[cfg(feature = "usb-ctap")]
    let ctap = Ctap::new(HIDClass::new(usb_bus, CTAP_REPORT_DESCRIPTOR, 5));
    let device = wasefire_board_api::platform::usb_device::<_, crate::board::Board>(usb_bus);
    State {
        protocol,
        #[cfg(feature = "usb-ctap")]
        ctap,
        device,
    }
}

pub enum Impl {}

#[cfg(feature = "_usb")]
impl Api for Impl {
    #[cfg(feature = "usb-ctap")]
    type Ctap = WithHid<Impl>;
}

#[cfg(feature = "usb-ctap")]
impl HasHid for Impl {
    type UsbBus = Usb;

    fn with_hid<R>(f: impl FnOnce(&mut Ctap<Self::UsbBus>) -> R) -> R {
        with_state(|state| f(&mut state.usb.ctap))
    }
}

impl HasRpc<'static, Usb> for Impl {
    fn with_rpc<R>(f: impl FnOnce(&mut Rpc<'static, Usb>) -> R) -> R {
        with_state(|state| f(&mut state.usb.protocol))
    }

    #[cfg(not(feature = "test-vendor"))]
    fn vendor(_: &[u8]) -> Result<Box<[u8]>, Error> {
        Err(Error::user(Code::NotImplemented))
    }

    #[cfg(feature = "test-vendor")]
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
        } else if let Some(request) = request.strip_prefix(b"next_boot ") {
            use wasefire_common::platform::Side;
            fn slot(x: u8) -> Option<Option<Side>> {
                Some(match x {
                    b'_' | b'U' => None,
                    b'A' => Some(Side::A),
                    b'B' => Some(Side::B),
                    _ => return None,
                })
            }
            let result = try {
                let (next, primary) = match request.trim_ascii_end() {
                    [next, b' ', primary] => (*next, *primary),
                    _ => None?,
                };
                (slot(next)?, slot(primary)?)
            };
            let Some((next, primary)) = result else {
                return Err(Error::user(Code::InvalidArgument));
            };
            crate::bootsvc::next_boot(next, primary)?;
            Ok(Box::default())
        } else {
            Err(Error::user(Code::InvalidArgument))
        }
    }
}

pub fn interrupt() {
    with_state(|state| {
        let mut classes: [&mut dyn UsbClass<_>; _] = [
            &mut state.usb.protocol,
            #[cfg(feature = "usb-ctap")]
            state.usb.ctap.class(),
        ];
        let _polled = state.usb.device.poll(&mut classes);
        state.usb.protocol.tick(|event| state.events.push(event.into()));
        #[cfg(feature = "usb-ctap")]
        state.usb.ctap.tick(|event| state.events.push(event.into()));
    });
}
