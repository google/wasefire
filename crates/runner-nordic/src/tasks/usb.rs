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

use board::usb::serial::Serial;
use nrf52840_hal::usbd::{UsbPeripheral, Usbd};

pub type Usb = Usbd<UsbPeripheral<'static>>;

impl board::usb::Api for crate::tasks::Board {}

impl board::usb::serial::HasSerial for crate::tasks::Board {
    type UsbBus = Usb;

    fn with_serial<R>(&mut self, f: impl FnOnce(&mut Serial<Self::UsbBus>) -> R) -> R {
        critical_section::with(|cs| f(&mut self.0.borrow_ref_mut(cs).serial))
    }
}
