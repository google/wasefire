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
//! The applet exposes statistics about BLE advertisement packets to the platform protocol.

#![no_std]
wasefire::applet!();

use alloc::collections::btree_map::BTreeMap;
use alloc::vec::Vec;
use core::cell::Cell;

use ble_adv::Advertisement;
use bytemuck::Zeroable;
use wasefire::platform::protocol::RpcProtocol;
use wasefire::rpc::Rpc;
use wasefire::sync::Mutex;
use wasefire::vendor::syscall;

const SYSCALL_ID: usize = ble_adv::SYSCALL_ID as usize;
const OP_REGISTER: usize = ble_adv::OP_REGISTER as usize;
const OP_READ_PACKET: usize = ble_adv::OP_READ_PACKET as usize;

fn main() {
    let rpc = RpcProtocol;
    let rpc_request = Cell::new(false);
    let ble_packet = Cell::new(true);
    let mut wait = None;
    STATS.lock().start = clock::uptime_us().unwrap();
    // Listen for RPC and BLE events.
    unsafe { rpc.register(set_cell, cell_ptr(&rpc_request)) }.unwrap();
    unsafe { ble_register(set_cell, cell_ptr(&ble_packet)) }.unwrap();
    loop {
        scheduling::wait_until(|| rpc_request.get() || ble_packet.get());
        if let Some(wait) = wait.take() {
            STATS.lock().wait += (clock::uptime_us().unwrap() - wait) as u32;
        }
        if let Some(request) = rpc.read().unwrap() {
            rpc_request.set(false);
            rpc.write(&process_rpc(request)).unwrap();
        }
        match ble_read().unwrap() {
            None => {
                ble_packet.set(false);
                wait = Some(clock::uptime_us().unwrap());
            }
            Some(packet) => process_ble(packet),
        }
    }
}

static STATS: Mutex<Stats> = Mutex::new(Stats { start: 0, wait: 0, stats: BTreeMap::new() });
struct Stats {
    start: u64,
    wait: u32,
    stats: BTreeMap<[u8; 6], Stat>,
}
#[derive(Default)]
struct Stat {
    rssi: i32,
    count: u32,
}

fn process_rpc(_request: Vec<u8>) -> Vec<u8> {
    let mut response = Vec::new();
    let end = clock::uptime_us().unwrap();
    let (start, wait, stats) = {
        let mut stats = STATS.lock();
        let start = core::mem::replace(&mut stats.start, end);
        let wait = core::mem::take(&mut stats.wait);
        let stats = core::mem::take(&mut stats.stats);
        (start, wait, stats)
    };
    response.extend_from_slice(&start.to_be_bytes());
    response.extend_from_slice(&end.to_be_bytes());
    response.extend_from_slice(&wait.to_be_bytes());
    for (addr, Stat { rssi, count }) in stats {
        response.extend_from_slice(&addr);
        response.extend_from_slice(&rssi.to_be_bytes());
        response.extend_from_slice(&count.to_be_bytes());
    }
    response
}

fn process_ble(packet: Advertisement) {
    let mut stats = STATS.lock();
    let stat = stats.stats.entry(packet.addr).or_default();
    stat.rssi += packet.rssi as i32;
    stat.count += 1;
}

fn cell_ptr(cell: &Cell<bool>) -> *const u8 {
    cell as *const _ as *const u8
}

extern "C" fn set_cell(cell: *const u8) {
    unsafe { &*(cell as *const Cell<bool>) }.set(true);
}

unsafe fn ble_register(func: extern "C" fn(*const u8), data: *const u8) -> Result<(), Error> {
    unsafe { syscall(SYSCALL_ID, OP_REGISTER, func as usize, data.addr()) }.map(|_| ())
}

fn ble_read() -> Result<Option<Advertisement>, Error> {
    let mut packet = Advertisement::zeroed();
    match unsafe { syscall(SYSCALL_ID, OP_READ_PACKET, &mut packet as *mut _ as usize, 0) } {
        Ok(0) => Ok(None),
        Ok(1) => Ok(Some(packet)),
        Ok(x) => panic!("platform returned Ok({x})"),
        Err(e) => Err(e),
    }
}
