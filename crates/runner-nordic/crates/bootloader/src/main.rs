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

#![no_std]
#![no_main]

use cortex_m::asm;
use cortex_m_rt::entry;
use embedded_storage::nor_flash::NorFlash;
use header::Header;
use nrf52840_hal::nvmc::Nvmc;
use nrf52840_hal::pac::{NVMC, Peripherals};
use panic_abort as _;
use wasefire_common::platform::Side;
use wasefire_one_of::exactly_one_of;

exactly_one_of!["board-devkit", "board-dongle", "board-makerdiary"];

#[entry]
fn main() -> ! {
    let mut new = Header::new(Side::A);
    if cfg!(feature = "single-sided") {
        unsafe { asm::bootload(new.firmware() as *const u32) };
    }
    let mut old = Header::new(Side::B);
    if new.timestamp() < old.timestamp() {
        core::mem::swap(&mut new, &mut old);
    }
    // We always try to mark the newest side (or side A in case of equality). The user can make sure
    // the new firmware works before updating the other side. If it doesn't, it suffices to reboot
    // the device a few times to boot in the previous version. Eventually and after a successful
    // update, both sides have the same timestamp and side B is running, because side A has consumed
    // all its attempts.
    let cur = if mark(new) { new } else { old };
    unsafe { asm::bootload(cur.firmware() as *const u32) };
}

fn mark(header: Header) -> bool {
    if !header.has_firmware() {
        return false;
    }
    let attempt = match (0 .. 3).map(|i| header.attempt(i)).find(|x| x.free()) {
        Some(x) => x.addr(),
        None => return false,
    };
    const PAGE_SIZE: usize = <Nvmc<NVMC> as NorFlash>::ERASE_SIZE;
    let header = header.addr();
    assert_eq!(header % PAGE_SIZE as u32, 0);
    let storage = unsafe { core::slice::from_raw_parts_mut(header as *mut u8, PAGE_SIZE) };
    let mut nvmc = Nvmc::new(Peripherals::take().unwrap().NVMC, storage);
    nvmc.write(attempt - header, &[0; 4]).unwrap();
    true
}
