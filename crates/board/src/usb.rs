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

//! USB interface.

#[cfg(feature = "api-usb-ctap")]
pub mod ctap;
#[cfg(feature = "api-usb-serial")]
pub mod serial;

/// USB event.
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, PartialEq, Eq)]
pub enum Event {
    /// CTAP HID event.
    #[cfg(feature = "api-usb-ctap")]
    Ctap(ctap::Event),

    /// Serial event.
    #[cfg(feature = "api-usb-serial")]
    Serial(serial::Event),
}

impl<B: crate::Api> From<Event> for crate::Event<B> {
    fn from(event: Event) -> Self {
        crate::Event::Usb(event)
    }
}

/// USB interface.
pub trait Api: Send {
    /// CTAP HID interface.
    #[cfg(feature = "api-usb-ctap")]
    type Ctap: ctap::Api;

    /// USB serial interface.
    #[cfg(feature = "api-usb-serial")]
    type Serial: serial::Api;
}

/// CTAP HID interface.
#[cfg(feature = "api-usb-ctap")]
pub type Ctap<B> = <super::Usb<B> as Api>::Ctap;

/// USB serial interface.
#[cfg(feature = "api-usb-serial")]
pub type Serial<B> = <super::Usb<B> as Api>::Serial;
