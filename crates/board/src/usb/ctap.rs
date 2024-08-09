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

use ctap_hid_fido2::{Cfg, FidoKeyHid, HidParam};
use usb_device::device::{Config, UsbDevice};
use usbip_device::UsbIpBus;

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
    fn read(output: &mut [u8; 64]) -> Result<usize, Error>;

    /// Writes from a buffer to the USB serial.
    ///
    /// Returns the number of bytes written. It could be zero if the other side is not ready.
    fn write(input: &[u8; 64]) -> Result<usize, Error>;

    /// Enables a given event to be triggered.
    fn enable(event: &Event) -> Result<(), Error>;

    /// Disables a given event from being triggered.
    fn disable(event: &Event) -> Result<(), Error>;
}

// Helper trait for boards using the `ctap_hid_fido2` crate.
pub trait HasCtapHid: Send {
    type UsbDevice: UsbDevice<'static, UsbIpBus>;

    fn with_ctaphid<R>(f: impl FnOnce(&mut CtapHid<Self::Hid>) -> R) -> R;
}

/// Wrapper type for boards using the `ctap_hid_fido2` crate.
pub struct WithCtapHid<T: HasCtapHid> {
    _never: !,
    _has_ctap: T,
}

/// Helper struct for boards using the `ctap_hid_fido2` crate.
pub struct CtapHid {
    ctap_hid: ctap_hid_fido2::FidoKeyHid,
    read_enabled: bool,
    write_enabled: bool,
}

impl CtapHid {
    pub fn new(config: Config) -> Self {
        params = HidParam::VidPid { vid: config.vendor_id, pid: config.product_id };
        ctap_hid = FidoKeyHid::new(params, &Cfg::init());
        Self { ctap_hid, read_enabled: false, write_enabled: false }
    }

    pub fn set(&mut self, event: &Event, enabled: bool) {
        match event {
            Event::Read => self.read_enabled = enabled,
            Event::Write => self.write_enabled = enabled,
        }
    }
}

impl<T: HasCtapHid> Api for WithCtapHid<T> {
    fn read(output: &mut [u8; 64]) -> Result<usize, Error> {
        match T::with_ctaphid(|ctap_hid| ctap_hid.read()) {
            Ok(len) => {
                log::trace!("{}{:?} = read({})", len, &output[.. len], output.len());
                Ok(len)
            }
            Err(e) => {
                log::debug!("{} = read({})", log::Debug2Format(&e), output.len());
                Err(Error::world(0))
            }
        }
    }

    fn write(input: &[u8; 64]) -> Result<usize, Error> {
        match T::with_ctaphid(|ctap_hid| ctap_hid.write(input)) {
            Ok(len) => {
                log::trace!("{} = write({}{:?})", len, input.len(), input);
                Ok(len)
            }
            Err(e) => {
                log::debug!("{} = write({}{:?})", log::Debug2Format(&e), input.len(), input);
                Err(Error::world(0))
            }
        }
    }

    fn enable(event: &Event) -> Result<(), Error> {
        T::with_ctaphid(|ctap_hid| ctap_hid.set(event, true));
        Ok(())
    }

    fn disable(event: &Event) -> Result<(), Error> {
        T::with_ctaphid(|ctap_hid| ctap_hid.set(event, false));
        Ok(())
    }
}
