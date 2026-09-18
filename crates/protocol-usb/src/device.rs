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
use alloc::collections::VecDeque;
use alloc::vec::Vec;
use core::marker::PhantomData;

use usb_device::class_prelude::{
    ControlIn, ControlOut, InterfaceNumber, StringIndex, UsbBus, UsbBusAllocator, UsbClass,
};
use usb_device::descriptor::{BosWriter, DescriptorWriter};
use usb_device::endpoint::{EndpointAddress, EndpointIn, EndpointOut};
use usb_device::{LangID, UsbError};
use wasefire_board_api::Error;
use wasefire_board_api::platform::protocol::{Api, Event};
use wasefire_error::Code;
use wasefire_logger as log;

use crate::common::{Decoder, Encoder};

pub struct Impl<'a, B: UsbBus, T: HasRpc<'a, B>> {
    _never: !,
    _phantom: PhantomData<(&'a (), B, T)>,
}

pub trait HasRpc<'a, B: UsbBus> {
    fn with_rpc<R>(f: impl FnOnce(&mut Rpc<'a, B>) -> R) -> R;
    fn vendor(request: &[u8]) -> Result<Box<[u8]>, Error>;
}

impl<'a, B: UsbBus, T: HasRpc<'a, B>> Api for Impl<'a, B, T> {
    fn read() -> Result<Option<Box<[u8]>>, Error> {
        T::with_rpc(|x| x.read())
    }

    fn write(response: &[u8]) -> Result<(), Error> {
        T::with_rpc(|x| x.write(response))
    }

    fn enable() -> Result<(), Error> {
        T::with_rpc(|x| x.enable())
    }

    fn vendor(request: &[u8]) -> Result<Box<[u8]>, Error> {
        T::vendor(request)
    }
}

pub struct Rpc<'a, B: UsbBus> {
    interface: InterfaceNumber,
    read_ep: EndpointOut<'a, B>,
    write_ep: EndpointIn<'a, B>,
    state: State,
}

impl<'a, B: UsbBus> Rpc<'a, B> {
    pub fn new(usb_bus: &'a UsbBusAllocator<B>) -> Self {
        let interface = usb_bus.interface();
        let read_ep = usb_bus.bulk(MAX_PACKET_SIZE);
        let write_ep = usb_bus.bulk(MAX_PACKET_SIZE);
        Rpc { interface, read_ep, write_ep, state: State::Disabled }
    }

    pub fn read(&mut self) -> Result<Option<Box<[u8]>>, Error> {
        let result = self.state.read()?;
        match &result {
            #[cfg(not(feature = "defmt"))]
            Some(result) => log::debug!("Reading {:02x?}", result),
            #[cfg(feature = "defmt")]
            Some(result) => log::debug!("Reading {=[u8]:02x}", result),
            None => log::debug!("Reading (no message)"),
        }
        Ok(result)
    }

    pub fn write(&mut self, response: &[u8]) -> Result<(), Error> {
        #[cfg(not(feature = "defmt"))]
        log::debug!("Writing {:02x?}", response);
        #[cfg(feature = "defmt")]
        log::debug!("Writing {=[u8]:02x}", response);
        self.state.write(response, &self.write_ep)
    }

    pub fn enable(&mut self) -> Result<(), Error> {
        match self.state {
            State::Disabled => {
                self.state = WaitRequest;
                Ok(())
            }
            _ => Err(Error::user(Code::InvalidState)),
        }
    }

    pub fn tick(&mut self, push: impl FnOnce(Event)) {
        if self.state.notify() {
            push(Event);
        }
    }
}

const MAX_PACKET_SIZE: u16 = 64;

enum State {
    Disabled,
    WaitRequest,
    ReceiveRequest { decoder: Decoder },
    RequestReady { notified: bool, request: Vec<u8> },
    WaitResponse,
    SendResponse { packets: VecDeque<[u8; 64]> },
}
use State::*;

impl State {
    fn read(&mut self) -> Result<Option<Box<[u8]>>, Error> {
        match self {
            RequestReady { request, .. } => {
                let request = core::mem::take(request);
                log::debug!("Received a message of {} bytes.", request.len());
                *self = WaitResponse;
                Ok(Some(request.into_boxed_slice()))
            }
            WaitRequest | ReceiveRequest { .. } | SendResponse { .. } => Ok(None),
            WaitResponse | Disabled => Err(Error::user(Code::InvalidState)),
        }
    }

    fn write<B: UsbBus>(&mut self, response: &[u8], ep: &EndpointIn<B>) -> Result<(), Error> {
        if !matches!(self, WaitResponse) {
            return Err(Error::user(Code::InvalidState));
        }
        let packets: VecDeque<_> = Encoder::new(response).collect();
        log::debug!("Sending a message of {} bytes in {} packets.", response.len(), packets.len());
        *self = SendResponse { packets };
        self.send(ep);
        Ok(())
    }

    fn receive<B: UsbBus>(&mut self, ep: &EndpointOut<B>) {
        let decoder = match self {
            ReceiveRequest { decoder } => decoder,
            Disabled => {
                log::error!("Not receiving data while disabled.");
                return;
            }
            _ => {
                *self = ReceiveRequest { decoder: Decoder::default() };
                match self {
                    ReceiveRequest { decoder } => decoder,
                    _ => unreachable!(),
                }
            }
        };
        let mut packet = [0; MAX_PACKET_SIZE as usize];
        let len = ep.read(&mut packet).unwrap();
        if len != MAX_PACKET_SIZE as usize {
            log::warn!("Received a packet of {} bytes instead of 64.", len);
            *self = WaitRequest;
            return;
        }
        match core::mem::take(decoder).push(&packet) {
            None => {
                log::warn!("Received invalid packet 0x{:02x}", packet[0]);
                *self = WaitRequest;
            }
            Some(Ok(request)) => {
                log::trace!("Received a message of {} bytes.", request.len());
                *self = RequestReady { notified: false, request };
            }
            Some(Err(x)) => {
                log::trace!("Received a packet.");
                *decoder = x;
            }
        }
    }

    fn send<B: UsbBus>(&mut self, ep: &EndpointIn<B>) {
        let packets = match self {
            Disabled => {
                log::error!("Not sending data while disabled.");
                return;
            }
            SendResponse { packets } => packets,
            _ => return,
        };
        let packet = match packets.pop_front() {
            Some(x) => x,
            None => {
                log::warn!("Invalid state: SendResponse with no packets.");
                *self = WaitRequest;
                return;
            }
        };
        let len = match ep.write(&packet) {
            Err(UsbError::WouldBlock) => {
                log::warn!("Failed to send packet, retrying later.");
                packets.push_front(packet);
                return;
            }
            x => x.unwrap(),
        };
        if len != MAX_PACKET_SIZE as usize {
            log::warn!("Sent a packet of {} bytes instead of 64.", len);
            *self = WaitRequest;
            return;
        }
        let remaining = packets.len();
        if packets.is_empty() {
            *self = WaitRequest;
        }
        log::trace!("Sent the next packet ({} remaining).", remaining);
    }

    fn notify(&mut self) -> bool {
        match self {
            RequestReady { notified, .. } => !core::mem::replace(notified, true),
            _ => false,
        }
    }
}

impl<B: UsbBus> UsbClass<B> for Rpc<'_, B> {
    fn get_configuration_descriptors(
        &self, writer: &mut DescriptorWriter,
    ) -> usb_device::Result<()> {
        writer.iad(self.interface, 1, 0xff, 0x58, 0x01, None)?;
        writer.interface(self.interface, 0xff, 0x58, 0x01)?;
        writer.endpoint(&self.write_ep)?;
        writer.endpoint(&self.read_ep)?;
        Ok(())
    }

    fn get_bos_descriptors(&self, writer: &mut BosWriter) -> usb_device::Result<()> {
        const PLATFORM: u8 = 0x05; // bDevCapabilityType
        writer.capability(PLATFORM, &WEBUSB_BOS_CAPABILITY)?;
        writer.capability(PLATFORM, &WINUSB_BOS_CAPABILITY)?;
        Ok(())
    }

    fn get_string(&self, _: StringIndex, _id: LangID) -> Option<&str> {
        // We don't have strings.
        None
    }

    fn reset(&mut self) {
        self.state = match self.state {
            State::Disabled => State::Disabled,
            _ => State::WaitRequest,
        };
    }

    fn poll(&mut self) {
        // We probably don't need to do anything here.
    }

    fn control_out(&mut self, _: ControlOut<B>) {
        // We probably don't need to do anything here.
    }

    fn control_in(&mut self, xfer: ControlIn<B>) {
        let req = xfer.request();
        if req.request_type != usb_device::control::RequestType::Vendor
            || req.recipient != usb_device::control::Recipient::Device
        {
            return;
        }
        match req.request {
            WEBUSB_VENDOR_CODE => {
                // Stall on invalid requests.
                let Some(descriptor) = WEBUSB_URL_DESC else { return xfer.reject().unwrap() };
                const GET_URL: u16 = 2;
                if req.index != GET_URL || req.value != 1 {
                    return xfer.reject().unwrap();
                }
                xfer.accept_with_static(descriptor).unwrap();
            }
            WINUSB_VENDOR_CODE => {
                const MS_OS_20_DESCRIPTOR_INDEX: u16 = 7;
                if req.index != MS_OS_20_DESCRIPTOR_INDEX || req.value != 0 {
                    return xfer.reject().unwrap();
                }
                let descriptor = make_winusb_desc(self.interface.into());
                xfer.accept_with(&descriptor).unwrap();
            }
            _ => (),
        }
    }

    fn endpoint_setup(&mut self, _: EndpointAddress) {
        // We probably don't need to do anything here.
    }

    fn endpoint_out(&mut self, addr: EndpointAddress) {
        if self.read_ep.address() != addr {
            return;
        }
        self.state.receive(&self.read_ep);
    }

    fn endpoint_in_complete(&mut self, addr: EndpointAddress) {
        if self.write_ep.address() != addr {
            return;
        }
        self.state.send(&self.write_ep);
    }
}

const WEBUSB_VENDOR_CODE: u8 = 1;
const WINUSB_VENDOR_CODE: u8 = 2;
const WINUSB_DESC_LEN: usize = 182;

macro_rules! make_descriptor {
    ($([$($x:expr),*$(,)?]),*$(,)?) => { [$($($x,)*)*] };
}

const WEBUSB_BOS_CAPABILITY: [u8; 21] = make_descriptor!(
    // bReserved
    [0x00],
    // PlatformCapabilityUUID
    [
        0x38, 0xb6, 0x08, 0x34, 0xa9, 0x09, 0xa0, 0x47, 0x8b, 0xfd, 0xa0, 0x76, 0x88, 0x15, 0xb6,
        0x65
    ],
    // bcdVersion
    [0x00, 0x01],
    // bVendorCode
    [WEBUSB_VENDOR_CODE],
    // iLandingPage
    [WEBUSB_URL_DESC.is_some() as u8],
);

const WINUSB_BOS_CAPABILITY: [u8; 25] = make_descriptor!(
    // bReserved
    [0x00],
    // PlatformCapabilityUUID (D8DD60DF-4589-4CC7-9CD2-659D9E648A9F)
    [
        0xdf, 0x60, 0xdd, 0xd8, 0x89, 0x45, 0xc7, 0x4c, 0x9c, 0xd2, 0x65, 0x9d, 0x9e, 0x64, 0x8a,
        0x9f
    ],
    // dwWindowsVersion (0x06030000 = Windows 8.1+)
    [0x00, 0x00, 0x03, 0x06],
    // wMSOSDescriptorSetTotalLength (182)
    [0xb6, 0x00],
    // bMS_VendorCode
    [WINUSB_VENDOR_CODE],
    // bAltEnumCode
    [0x00],
);

const fn make_winusb_desc(interface: u8) -> [u8; WINUSB_DESC_LEN] {
    make_descriptor!(
        // Microsoft OS 2.0 descriptor set header (10 bytes)
        [0x0a, 0x00],             // wLength (10)
        [0x00, 0x00],             // wDescriptorType (MS_OS_20_SET_HEADER_DESCRIPTOR = 0)
        [0x00, 0x00, 0x03, 0x06], // dwWindowsVersion (0x06030000 = Windows 8.1+)
        [0xb6, 0x00],             // wTotalLength (182)
        // Microsoft OS 2.0 CCGP device descriptor (4 bytes)
        [0x04, 0x00], // wLength (4)
        [0x07, 0x00], // wDescriptorType (MS_OS_20_FEATURE_CCGP_DEVICE = 7)
        // Microsoft OS 2.0 configuration subset header (8 bytes)
        [0x08, 0x00], // wLength (8)
        [0x01, 0x00], // wDescriptorType (MS_OS_20_SUBSET_HEADER_CONFIGURATION = 1)
        [0x00],       // bConfigurationValue (0)
        [0x00],       // bReserved
        [0xa8, 0x00], // wTotalLength (168)
        // Microsoft OS 2.0 function subset header (8 bytes)
        [0x08, 0x00], // wLength (8)
        [0x02, 0x00], // wDescriptorType (MS_OS_20_SUBSET_HEADER_FUNCTION = 2)
        [interface],  // bFirstInterface
        [0x00],       // bReserved
        [0xa0, 0x00], // wSubsetLength (160)
        // Microsoft OS 2.0 compatible ID descriptor (20 bytes)
        [0x14, 0x00],                                     // wLength (20)
        [0x03, 0x00], // wDescriptorType (MS_OS_20_FEATURE_COMPATBLE_ID = 3)
        [b'W', b'I', b'N', b'U', b'S', b'B', 0x00, 0x00], // CompatibleID ("WINUSB")
        [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00], // SubCompatibleID
        // Microsoft OS 2.0 registry property descriptor (132 bytes)
        [0x84, 0x00], // wLength (132)
        [0x04, 0x00], // wDescriptorType (MS_OS_20_FEATURE_REG_PROPERTY = 4)
        [0x07, 0x00], // wPropertyDataType (REG_MULTI_SZ = 7)
        [0x2a, 0x00], // wPropertyNameLength (42)
        // PropertyName ("DeviceInterfaceGUIDs\0" in UTF-16LE)
        [
            b'D', 0, b'e', 0, b'v', 0, b'i', 0, b'c', 0, b'e', 0, b'I', 0, b'n', 0, b't', 0, b'e',
            0, b'r', 0, b'f', 0, b'a', 0, b'c', 0, b'e', 0, b'G', 0, b'U', 0, b'I', 0, b'D', 0,
            b's', 0, 0, 0
        ],
        [0x50, 0x00], // wPropertyDataLength (80)
        // PropertyData ("{b649b634-31ec-4394-ba28-42324d6c3993}\0\0" in UTF-16LE)
        [
            b'{', 0, b'b', 0, b'6', 0, b'4', 0, b'9', 0, b'b', 0, b'6', 0, b'3', 0, b'4', 0, b'-',
            0, b'3', 0, b'1', 0, b'e', 0, b'c', 0, b'-', 0, b'4', 0, b'3', 0, b'9', 0, b'4', 0,
            b'-', 0, b'b', 0, b'a', 0, b'2', 0, b'8', 0, b'-', 0, b'4', 0, b'2', 0, b'3', 0, b'2',
            0, b'4', 0, b'd', 0, b'6', 0, b'c', 0, b'3', 0, b'9', 0, b'9', 0, b'3', 0, b'}', 0, 0,
            0, 0, 0
        ],
    )
}

const WEBUSB_URL_DESC: Option<&[u8]> = {
    const SPLIT: (Option<u8>, &[u8]) = split_webusb_url(option_env!("WASEFIRE_WEBUSB_URL"));
    match SPLIT.0 {
        None => None,
        Some(scheme) => Some(&make_webusb_url::<{ 3 + SPLIT.1.len() }>(scheme, SPLIT.1)),
    }
};

const fn make_webusb_url<const LEN: usize>(scheme: u8, data: &[u8]) -> [u8; LEN] {
    assert!(LEN < 256);
    let mut result = [0; LEN];
    result[0] = LEN as u8; // bLength
    result[1] = 3; // bDescriptorType = WEBUSB_URL
    result[2] = scheme; // bScheme
    let mut i = 0;
    while 3 + i < LEN {
        result[3 + i] = data[i];
        i += 1;
    }
    result
}

const fn split_webusb_url(url: Option<&'static str>) -> (Option<u8>, &'static [u8]) {
    let Some(url) = url else { return (None, &[]) };
    let url = url.as_bytes();
    let (scheme, data) = if let Some(data) = strip_prefix(url, b"http://") {
        (0, data)
    } else if let Some(data) = strip_prefix(url, b"https://") {
        (1, data)
    } else {
        (255, url)
    };
    (Some(scheme), data)
}

const fn strip_prefix(data: &'static [u8], prefix: &'static [u8]) -> Option<&'static [u8]> {
    if data.len() < prefix.len() {
        return None;
    }
    let mut i = 0;
    while i < prefix.len() {
        if data[i] != prefix[i] {
            return None;
        }
        i += 1;
    }
    let ptr = unsafe { data.as_ptr().add(prefix.len()) };
    let len = data.len() - prefix.len();
    Some(unsafe { core::slice::from_raw_parts(ptr, len) })
}
