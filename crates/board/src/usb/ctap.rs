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
use usbd_ctaphid::CtapHid;

use super::serial::{self, HasSerial};

/// CTAP event.
#[derive(Debug, PartialEq, Eq)]
pub enum Event {
    /// There might be data to read.
    ///
    /// This is only guaranteed to be triggered after a short read.
    Read,

    /// It might be possible to write data.
    ///
    /// This is only guaranteed to be triggered after a short write.
    Write,
}

impl<B: crate::Api> From<Event> for crate::Event<B> {
    fn from(event: Event) -> Self {
        super::Event::Ctap(event).into()
    }
}

pub trait Api: Send {
    /// Reads from the USB serial into a buffer.
    ///
    /// Returns the number of bytes read. It could be zero if there's nothing to read.
    fn read(output: &mut [u8]) -> Result<usize, Error>;

    /// Writes from a buffer to the USB serial.
    ///
    /// Returns the number of bytes written. It could be zero if the other side is not ready.
    fn write(input: &[u8]) -> Result<usize, Error>;
}

pub trait HasCtapHid: Send {
    type UsbBus: UsbBus;

    fn with_ctaphid<R>(f: impl FnOnce(&mut CtapHid<Self::UsbBus>) -> R) -> R;
}

/// Wrapper type for boards using the `usbd_ctaphid` crate.
pub struct WithCtapHid<T: HasCtapHid> {
    _never: !,
    _has_serial: T,
}

/// Helper struct for boards using the `usbd_ctaphid` crate.
pub struct CtapHid<'a, T: UsbBus> {
    ctap_hid: usbd_ctaphid::CtapHid,
    read_enabled: bool,
    write_enabled: bool,
}

impl<T: HasCtapHid> Api for WithCtapHid<T> {
    fn read(output: &mut [u8]) -> Result<usize, Error> {
        match T::with_ctaphid(|ctap_hid| ctap_hid.pipe.read_and_handle_packet()) {
          // todo!()
        }
    }

    fn write(input: &[u8]) -> Result<usize, Error> {
        match T::with_ctaphid(|ctap_hid| ctap_hid.check_for_app_response()) {
          // todo!()
        }
    }
}
