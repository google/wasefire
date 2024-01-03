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

//! Radio interface.

use crate::Unsupported;

#[cfg(feature = "api-radio-ble")]
pub mod ble;

/// Radio event.
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Event {
    /// BLE event.
    #[cfg(feature = "api-radio-ble")]
    Ble(ble::Event),
}

impl<B: crate::Api> From<Event> for crate::Event<B> {
    fn from(event: Event) -> Self {
        crate::Event::Radio(event)
    }
}

/// Radio interface.
pub trait Api: Send {
    #[cfg(feature = "api-radio-ble")]
    type Ble: ble::Api;
}

#[cfg(feature = "api-radio-ble")]
pub type Ble<B> = <super::Radio<B> as Api>::Ble;

impl Api for Unsupported {
    #[cfg(feature = "api-radio-ble")]
    type Ble = Unsupported;
}
