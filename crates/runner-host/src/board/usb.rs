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
use usbip_device::UsbIpBus;

impl board::usb::Api for crate::board::Board {}

impl board::usb::serial::HasSerial for crate::board::Board {
    type UsbBus = UsbIpBus;

    fn with_serial<R>(&mut self, f: impl FnOnce(&mut Serial<Self::UsbBus>) -> R) -> R {
        f(&mut self.state.lock().unwrap().serial)
    }
}
