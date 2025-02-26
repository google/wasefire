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

//! CTAP HID interface.

use usb_device::bus::UsbBus;
use usbd_hid::UsbError;
use usbd_hid::hid_class::HIDClass;
use wasefire_error::Code;
use {ssmarshal as _, wasefire_logger as log};

use crate::Error;

/// CTAP HID event.
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, PartialEq, Eq)]
pub enum Event {
    /// There might be data to read.
    ///
    /// This is only guaranteed to be triggered after a failed read.
    Read,

    /// It might be possible to write data.
    ///
    /// This is only guaranteed to be triggered after a failed write.
    Write,
}

impl<B: crate::Api> From<Event> for crate::Event<B> {
    fn from(event: Event) -> Self {
        super::Event::Ctap(event).into()
    }
}

/// CTAP HID interface.
pub trait Api: Send {
    /// Reads a CTAP HID packet into a buffer.
    ///
    /// Returns whether a packet was read read.
    fn read(output: &mut [u8; 64]) -> Result<bool, Error>;

    /// Writes a CTAP HID packet from a buffer.
    ///
    /// Returns whether the packet was written.
    fn write(input: &[u8; 64]) -> Result<bool, Error>;

    /// Enables a given event to be triggered.
    fn enable(event: &Event) -> Result<(), Error>;

    /// Disables a given event from being triggered.
    fn disable(event: &Event) -> Result<(), Error>;
}

/// Helper trait for boards using the `usbd-hid` crate.
pub trait HasHid: Send {
    /// USB bus type.
    type UsbBus: UsbBus;

    /// Provides scoped access to the `Ctap` type.
    fn with_hid<R>(f: impl FnOnce(&mut Ctap<Self::UsbBus>) -> R) -> R;
}

/// Wrapper type for boards using the `usbd-hid` crate.
pub struct WithHid<T: HasHid> {
    _never: !,
    _has_hid: T,
}

/// Helper struct for boards using the `usbd-hid` crate.
pub struct Ctap<'a, T: UsbBus> {
    class: HIDClass<'a, T>,
    read_enabled: bool,
    write_enabled: bool,
}

impl<'a, T: UsbBus> Ctap<'a, T> {
    /// Creates a new serial from a port.
    pub fn new(class: HIDClass<'a, T>) -> Self {
        Self { class, read_enabled: false, write_enabled: false }
    }

    /// Accesses the HID class.
    pub fn class(&mut self) -> &mut HIDClass<'a, T> {
        &mut self.class
    }

    /// Pushes events based on whether the USB serial was polled.
    pub fn tick(&mut self, mut push: impl FnMut(Event)) {
        if self.read_enabled {
            push(Event::Read);
        }
        if self.write_enabled {
            push(Event::Write);
        }
    }

    fn set(&mut self, event: &Event, enabled: bool) -> Result<(), Error> {
        match event {
            Event::Read => self.read_enabled = enabled,
            Event::Write => self.write_enabled = enabled,
        }
        Ok(())
    }
}

impl<T: HasHid> Api for WithHid<T> {
    fn read(output: &mut [u8; 64]) -> Result<bool, Error> {
        T::with_hid(|x| match x.class.pull_raw_output(output) {
            Ok(64) => Ok(true),
            Ok(len) => {
                log::warn!("bad read len {}", len);
                Err(Error::world(Code::InvalidLength))
            }
            Err(UsbError::WouldBlock) => Ok(false),
            Err(_) => Err(Error::world(0)),
        })
    }

    fn write(input: &[u8; 64]) -> Result<bool, Error> {
        T::with_hid(|x| match x.class.push_raw_input(input) {
            Ok(64) => Ok(true),
            Ok(len) => {
                log::warn!("bad write len {}", len);
                Err(Error::world(Code::InvalidLength))
            }
            Err(UsbError::WouldBlock) => Ok(false),
            Err(_) => Err(Error::world(0)),
        })
    }

    fn enable(event: &Event) -> Result<(), Error> {
        T::with_hid(|x| x.set(event, true))
    }

    fn disable(event: &Event) -> Result<(), Error> {
        T::with_hid(|x| x.set(event, false))
    }
}
