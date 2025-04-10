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
// limitations under the License.

use earlgrey::FLASH_CTRL_CORE as FLASH_CTRL;
use wasefire_common::platform::Side;
use wasefire_error::{CodeParam, Error};
use wasefire_logger as log;

use crate::multibit::BOOL4_FALSE;

pub fn init() {
    #[cfg(any())]
    dump_regions();

    // After ROM_EXT on bank A, the flash regions look like this:
    // 0: 09669966 00000000-0000ffff RD            SCRAMBLE ECC    LOCKED
    // 1: 09666666 00080000-0008ffff RD PROG ERASE SCRAMBLE ECC    LOCKED
    // 2: 09669966 00010000-00076fff RD            SCRAMBLE ECC    LOCKED
    // 3: 06696666 00077000-0007ffff RD PROG ERASE          ECC HE
    // 4: 09666666 00090000-000f6fff RD PROG ERASE SCRAMBLE ECC    LOCKED
    // 5: 06696666 000f7000-000ff7ff RD PROG ERASE          ECC HE
    // 6: 09669966 000ff800-000fffff RD            SCRAMBLE ECC    LOCKED
    // 7: 09999999 00000000-ffffffff -
    //
    // We disable ECC for the storage since we need to write multiple times to the same word. We
    // keep HE because it permits 100k erase cycles instead of 10k to the price of slower
    // operations.
    FLASH_CTRL.mp_region_cfg(3).modify().ecc_en(BOOL4_FALSE).reg.write();
    FLASH_CTRL.mp_region_cfg(5).modify().ecc_en(BOOL4_FALSE).reg.write();
}

pub fn erase(addr: usize) -> Result<(), Error> {
    if addr & (PAGE_SIZE - 1) != 0 {
        return Err(error(Code::EraseAlign));
    }
    if !(FLASH_BEG .. FLASH_END).contains(&addr) {
        return Err(error(Code::EraseBound));
    }
    FLASH_CTRL.addr().reset().start((addr - FLASH_BEG) as u32).reg.write();
    FLASH_CTRL.control().reset().start(true).op(2).reg.write();
    wait(Code::EraseError)
}

pub fn write(mut addr: usize, mut data: &[u8]) -> Result<(), Error> {
    const WORD_SIZE: usize = 4; // bus word, not flash word
    if addr & (WORD_SIZE - 1) != 0 || data.len() & (WORD_SIZE - 1) != 0 {
        return Err(error(Code::WriteAlign));
    }
    if addr < FLASH_BEG || addr.checked_add(data.len()).is_none_or(|x| FLASH_END < x) {
        return Err(error(Code::WriteBound));
    }
    while !data.is_empty() {
        let max_len = (addr / WINDOW_SIZE + 1) * WINDOW_SIZE - addr;
        let word_len = core::cmp::min(max_len, data.len()) / WORD_SIZE;
        FLASH_CTRL.addr().reset().start((addr - FLASH_BEG) as u32).reg.write();
        FLASH_CTRL.control().reset().start(true).op(1).num((word_len - 1) as u32).reg.write();
        for _ in 0 .. word_len {
            let mut word = [0; WORD_SIZE];
            word.copy_from_slice(&data[.. WORD_SIZE]);
            FLASH_CTRL.prog_fifo(0).write_raw(u32::from_le_bytes(word));
            addr += WORD_SIZE;
            data = &data[WORD_SIZE ..];
        }
        wait(Code::WriteError)?;
    }
    Ok(())
}

fn wait(code: Code) -> Result<(), Error> {
    const TIMEOUT: u64 = 100000; // 100ms
    let deadline = crate::time::uptime_us() + TIMEOUT;
    while crate::time::uptime_us() < deadline {
        let status = FLASH_CTRL.op_status().read();
        if status.err() {
            log::warn!(
                "flash operation failed {=u32:08x}@{=u32:08x}",
                FLASH_CTRL.err_code().read_raw(),
                FLASH_CTRL.err_addr().read_raw(),
            );
            FLASH_CTRL.op_status().reset().reg.write();
            return Err(error(code));
        }
        if status.done() {
            FLASH_CTRL.op_status().reset().reg.write();
            return Ok(());
        }
    }
    log::warn!("flash operation timed out");
    Err(error(Code::WaitLong))
}

pub fn active() -> Side {
    bank_side(crate::manifest::active() as *const _ as usize)
}

pub fn inactive() -> Side {
    active().opposite()
}

pub fn bank_side(addr: usize) -> Side {
    match addr & BANK_SIZE != 0 {
        false => Side::A,
        true => Side::B,
    }
}

pub fn flip_bank(addr: usize) -> usize {
    addr ^ BANK_SIZE
}

pub const PAGE_SIZE: usize = 0x800;
const BANK_SIZE: usize = 0x80000;
const FLASH_BEG: usize = 0x20000000;
const FLASH_END: usize = 0x20100000;
const WINDOW_SIZE: usize = 64;

// We rely on those in bank_side() and flip_bank().
const _: () = assert!(BANK_SIZE.is_power_of_two());
const _: () = assert!(FLASH_BEG + 2 * BANK_SIZE == FLASH_END);
const _: () = assert!(FLASH_BEG & (2 * BANK_SIZE - 1) == 0);

fn error(code: Code) -> Error {
    Error::new(crate::error::Space::Flash, code)
}
#[repr(u16)]
enum Code {
    EraseAlign = 0x8011,
    EraseBound = 0x8012,
    EraseError = 0x8013,
    WriteAlign = 0x8021,
    WriteBound = 0x8022,
    WriteError = 0x8023,
    WaitLong = 0x8031,
}

impl From<Code> for u16 {
    fn from(value: Code) -> Self {
        value as u16
    }
}

impl CodeParam for Code {}

#[cfg(any())]
fn dump_regions() {
    for i in 0 .. 8 {
        let cfg = FLASH_CTRL.mp_region_cfg(i).read_raw();
        let lock = !FLASH_CTRL.region_cfg_regwen(i).read().region() as u32;
        let reg = FLASH_CTRL.mp_region(i).read();
        let start = reg.base() * PAGE_SIZE as u32;
        let limit = start + reg.size() * PAGE_SIZE as u32 - 1;
        log::info!("{}: {:08x} {} {:08x}-{:08x}", i, cfg, lock, start, limit);
    }
}
