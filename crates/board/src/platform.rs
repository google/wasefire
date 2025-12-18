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

//! Platform interface.

use alloc::string::String;

use usb_device::bus::{UsbBus, UsbBusAllocator};
use usb_device::device::{StringDescriptors, UsbDevice, UsbDeviceBuilder, UsbVidPid};
use wasefire_sync::Once;

use crate::Error;

pub mod protocol;

/// Platform event.
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, PartialEq, Eq)]
pub enum Event {
    /// Protocol event.
    Protocol(protocol::Event),
}

impl<B: crate::Api> From<Event> for crate::Event<B> {
    fn from(event: Event) -> Self {
        crate::Event::Platform(event)
    }
}

/// Platform interface.
pub trait Api: Send {
    /// Platform protocol interface.
    type Protocol: protocol::Api;

    /// Platform update interface.
    ///
    /// Calling `finish()` will reboot if the update is successful and thus only returns in case of
    /// errors or in dry-run mode.
    type Update: crate::transfer::Api;

    /// Returns the platform serial.
    fn serial() -> alloc::borrow::Cow<'static, [u8]>;

    /// Returns the platform running side.
    fn running_side() -> wasefire_common::platform::Side;

    /// Returns the platform information of the running side.
    fn running_info() -> wasefire_protocol::platform::SideInfo0<'static>;

    /// Returns the platform information of the opposite side, if any.
    ///
    /// # Errors
    ///
    /// There is a precise meaning to the following errors:
    /// - `World:NotEnough`: This platform has only one side.
    /// - `World:NotFound`: The other side is empty.
    fn opposite_info() -> Result<wasefire_protocol::platform::SideInfo0<'static>, Error>;

    /// Reboots the device (thus platform and applets).
    fn reboot() -> Result<!, Error>;
}

/// Platform protocol interface.
pub type Protocol<B> = <super::Platform<B> as Api>::Protocol;

/// Platform update interface.
pub type Update<B> = <super::Platform<B> as Api>::Update;

/// Builds a Wasefire USB device.
///
/// The USB bus should have the Wasefire protocol registered. It may have additional USB classes.
pub fn usb_device<U: UsbBus, B: crate::Api>(usb_bus: &UsbBusAllocator<U>) -> UsbDevice<'_, U> {
    static SERIAL: Once<String> = Once::new();
    let serial = SERIAL.call_once(|| data_encoding::HEXLOWER.encode(&B::Platform::serial()));
    UsbDeviceBuilder::new(usb_bus, UsbVidPid(0x18d1, 0x0239))
        .strings(&[StringDescriptors::new(usb_device::LangID::EN_US)
            .manufacturer("Google Inc.")
            .product("Wasefire")
            .serial_number(serial)])
        .unwrap()
        .build()
}
