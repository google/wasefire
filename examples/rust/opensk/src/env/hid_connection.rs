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
// limitations under the License.use opensk_lib::api::clock::Clock;

use opensk_lib::api::connection::{HidConnection, RecvStatus, UsbEndpoint};
use opensk_lib::ctap::status_code::CtapResult;
use wasefire::scheduling;
use wasefire::usb::ctap;

use crate::env::{WasefireEnv, convert_error};

impl HidConnection for WasefireEnv {
    fn send(&mut self, packet: &[u8; 64], endpoint: UsbEndpoint) -> CtapResult<()> {
        let UsbEndpoint::MainHid = endpoint;
        let mut listener = ctap::Listener::new(ctap::Event::Write);
        while !ctap::write(packet).map_err(convert_error)? {
            scheduling::wait_until(|| listener.is_notified());
        }
        Ok(())
    }

    fn recv(&mut self, packet: &mut [u8; 64], timeout_ms: usize) -> CtapResult<RecvStatus> {
        let mut listener = ctap::Listener::new(ctap::Event::Read);
        let timeout = wasefire::timer::Timeout::new_ms(timeout_ms);
        loop {
            if ctap::read(packet).map_err(convert_error)? {
                return Ok(RecvStatus::Received(UsbEndpoint::MainHid));
            }
            scheduling::wait_until(|| timeout.is_over() || listener.is_notified());
            if timeout.is_over() {
                return Ok(RecvStatus::Timeout);
            }
        }
    }
}
