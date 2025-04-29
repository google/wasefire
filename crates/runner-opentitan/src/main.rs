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

#![no_std]
#![no_main]
#![feature(array_chunks)]
#![feature(never_type)]
#![feature(try_blocks)]

extern crate alloc;

use earlgrey::RSTMGR_AON;
#[cfg(feature = "wasm")]
use wasefire_interpreter as _;
use wasefire_logger as log;
use wasefire_one_of::exactly_one_of;
use wasefire_scheduler::Scheduler;

exactly_one_of!["debug", "release"];
exactly_one_of!["native", "wasm"];

mod allocator;
mod board;
mod bootsvc;
mod error;
mod flash;
mod hmac;
mod manifest;
mod multibit;
mod plic;
mod time;
#[cfg(feature = "debug")]
mod uart;
mod usb;
mod util;

fn idle() -> ! {
    loop {
        riscv::asm::wfi();
    }
}

fn reboot() -> ! {
    #[cfg(feature = "debug")]
    uart::reset();
    RSTMGR_AON.reset_req().reset().val(multibit::BOOL4_TRUE).reg.write();
    idle();
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    log::println!("{}", log::Display2Format(info));
    #[cfg(feature = "release")]
    reboot();
    #[cfg(feature = "debug")]
    uart::exit(false);
}

#[cfg(feature = "debug")]
defmt::timestamp!("{=u64:us}", time::uptime_us());

#[riscv_rt::entry]
fn main() -> ! {
    allocator::init();
    time::init();
    #[cfg(feature = "debug")]
    log::info!("{}", bootsvc::Format);
    flash::init();
    board::init();
    // Interrupts may assume the board is initialized, so this must be last.
    plic::init();
    log::debug!("Runner is initialized.");
    Scheduler::<board::Board>::run();
}
