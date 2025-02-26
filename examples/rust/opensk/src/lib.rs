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

//! Example applet running OpenSK.
//!
//! The Nordic runner needs the usb-ctap, software-crypto-aes256-cbc, software-crypto-p256, and
//! software-crypto-hmac-sha256 features.

#![no_std]
wasefire::applet!();

use opensk_lib::api::connection::HidConnection as _;
use opensk_lib::env::Env as _;

mod env;

fn main() -> ! {
    let mut opensk_ctap = opensk_lib::Ctap::new(env::init());
    let mut ctap_listener = usb::ctap::Listener::new(usb::ctap::Event::Read);
    let transport = opensk_lib::Transport::MainHid;
    let endpoint = opensk_lib::api::connection::UsbEndpoint::MainHid;
    debug!("OpenSK initialized");
    loop {
        let mut packet = [0; 64];
        while !usb::ctap::read(&mut packet).unwrap() {
            scheduling::wait_until(|| ctap_listener.is_notified());
        }
        for packet in opensk_ctap.process_hid_packet(&packet, transport) {
            opensk_ctap.env().hid_connection().send(&packet, endpoint).unwrap();
        }
    }
}
