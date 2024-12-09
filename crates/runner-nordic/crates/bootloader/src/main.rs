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
use header::{Header, Side};
use nrf52840_hal::nvmc::Nvmc;
use nrf52840_hal::pac::{NVMC, Peripherals};
use panic_abort as _;

#[entry]
fn main() -> ! {
    let mut new = Header::new(Side::A);
    let mut old = Header::new(Side::B);
    if new.timestamp() < old.timestamp() {
        core::mem::swap(&mut new, &mut old);
    }
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
