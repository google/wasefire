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

use alloc::string::String;

use interface::{deserialize, serialize, Request, Response};
use wasefire::serial;

/// Receives a request from the serial.
pub fn receive() -> Request {
    deserialize(|x| serial::read_any(&SERIAL, x).unwrap())
}

/// Sends a response to the serial.
pub fn send(response: Result<Response, String>) {
    serial::write_all(&SERIAL, &serialize(&response)).unwrap();
}

#[cfg(feature = "usb")]
type Serial = wasefire::usb::serial::UsbSerial;
#[cfg(feature = "usb")]
const SERIAL: Serial = wasefire::usb::serial::UsbSerial;
#[cfg(not(feature = "usb"))]
type Serial = wasefire::uart::Uart;
#[cfg(not(feature = "usb"))]
const SERIAL: Serial = wasefire::uart::Uart(0);
