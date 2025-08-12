// Copyright 2025 Google LLC
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

//! Demonstrates simple BLE advertisement usage for Nordic.
//!
//! The applet prints BLE advertisement packets to the debug output.

#![no_std]
wasefire::applet!();

use ble_adv::Advertisement;
use bytemuck::Zeroable;
use data_encoding::HEXLOWER;
use wasefire::vendor::syscall;

fn main() {
    unsafe { syscall(0x80000001, 1, print_event as usize, 0) }.unwrap();
}

extern "C" fn print_event(_: *const u8) {
    let mut packet = Advertisement::zeroed();
    match unsafe { syscall(0x80000001, 3, &mut packet as *mut _ as usize, 0) } {
        Ok(0) => debug!("no packet"),
        Ok(1) => {
            let freq = packet.freq;
            let rssi = packet.rssi;
            let pdu = packet.pdu_type;
            let addr = HEXLOWER.encode(&packet.addr);
            let data = HEXLOWER.encode(packet.data());
            debug!("{freq}MHz R{rssi} T{pdu} @{addr}: {data}");
        }
        Ok(x) => panic!("platform returned Ok({x})"),
        Err(error) => debug!("error: {error}"),
    }
}
