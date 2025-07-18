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
//! To build a native applet for the nRF52840 dongle:
//!
//! ```shell
//! cargo xtask --release --native \
//!   applet rust opensk --opt-level=z \
//!   runner nordic --board=dongle --opt-level=z \
//!     --features=usb-ctap --features=software-crypto-aes256-cbc \
//!     --features=software-crypto-p256,software-crypto-hmac-sha256 \
//!   flash
//! ```
//!
//! To build a native applet for the OpenTitan A2 board:
//!
//! ```shell
//! cargo xtask --release --native \
//!   applet rust opensk --opt-level=z \
//!   runner opentitan --opt-level=z --features=usb-ctap \
//!   flash
//! ```

#![no_std]
wasefire::applet!();

use opensk_lib::Transport;
use opensk_lib::api::connection::{HidConnection as _, RecvStatus, UsbEndpoint};
use opensk_lib::env::Env as _;

mod blink;
mod env;

fn main() -> ! {
    let mut opensk_ctap = opensk_lib::Ctap::new(env::init());
    let mut wink: Option<blink::Blink> = None;
    debug!("OpenSK initialized");
    loop {
        match (wink.is_some(), opensk_ctap.should_wink()) {
            (true, true) | (false, false) => (),
            (false, true) => wink = Some(blink::Blink::new_ms(100)),
            (true, false) => wink = None,
        }
        let mut packet = [0; 64];
        match opensk_ctap.env().hid_connection().recv(&mut packet, 100).unwrap() {
            RecvStatus::Timeout => continue,
            RecvStatus::Received(endpoint) => assert_eq!(endpoint, UsbEndpoint::MainHid),
        }
        for packet in opensk_ctap.process_hid_packet(&packet, Transport::MainHid) {
            opensk_ctap.env().hid_connection().send(&packet, UsbEndpoint::MainHid).unwrap();
        }
    }
}
