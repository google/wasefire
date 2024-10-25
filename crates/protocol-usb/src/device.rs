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
use wasefire_board_api::platform::protocol::{Api, Event};
use wasefire_board_api::Error;
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
        let packets: VecDeque<_> = Encoder::new(response).map(Into::into).collect();
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

impl<'a, B: UsbBus> UsbClass<B> for Rpc<'a, B> {
    fn get_configuration_descriptors(
        &self, writer: &mut DescriptorWriter,
    ) -> usb_device::Result<()> {
        writer.iad(self.interface, 1, 0xff, 0x58, 0x01, None)?;
        writer.interface(self.interface, 0xff, 0x58, 0x01)?;
        writer.endpoint(&self.write_ep)?;
        writer.endpoint(&self.read_ep)?;
        Ok(())
    }

    fn get_bos_descriptors(&self, _: &mut BosWriter) -> usb_device::Result<()> {
        // We don't have any capabilities.
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

    fn control_in(&mut self, _: ControlIn<B>) {
        // We probably don't need to do anything here.
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
