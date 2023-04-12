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

use board::usb::serial::{HasSerial, WithSerial};
use usbip_device::UsbIpBus;
use wasefire_board_api as board;
use wasefire_board_api::usb::serial::Serial;

use crate::board::Board;

impl board::usb::Api for &mut Board {
    type Serial<'a> = WithSerial<&'a mut Board>
    where Self: 'a;
    fn serial(&mut self) -> Self::Serial<'_> {
        WithSerial(self)
    }
}

impl HasSerial for &mut Board {
    type UsbBus = UsbIpBus;

    fn with_serial<R>(&mut self, f: impl FnOnce(&mut Serial<Self::UsbBus>) -> R) -> R {
        f(&mut self.state.lock().unwrap().serial)
    }
}
