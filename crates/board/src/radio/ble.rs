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

//! Bluetooth Low Energy (BLE) interface.

use wasefire_applet_api::radio::ble::Advertisement;

use crate::{Error, Support};

/// BLE event.
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Event {
    /// Received an advertisement packet.
    Advertisement,
}

impl<B: crate::Api> From<Event> for crate::Event<B> {
    fn from(event: Event) -> Self {
        super::Event::Ble(event).into()
    }
}

/// BLE interface.
pub trait Api: Support<bool> {
    /// Enables BLE events.
    fn enable(event: &Event) -> Result<(), Error>;

    /// Disables BLE events.
    fn disable(event: &Event) -> Result<(), Error>;

    /// Reads the next advertisement packet, if any.
    ///
    /// Returns whether a packet was read.
    fn read_advertisement(packet: &mut Advertisement) -> Result<bool, Error>;
}
