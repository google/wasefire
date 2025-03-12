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

use earlgrey::{RV_PLIC, plic};
use riscv::interrupt::{Exception, Interrupt, Trap, cause};
use wasefire_logger as log;

pub fn init() {
    enable(plic::RV_TIMER_MIN, plic::RV_TIMER_MAX);
    enable(plic::USBDEV_MIN, plic::USBDEV_MAX);
    RV_PLIC.threshold(0).reset().threshold(0).reg.write();
    unsafe {
        riscv::interrupt::enable();
        riscv::register::mie::set_mext();
    }
}

fn enable(min: u32, max: u32) {
    for i in min ..= max {
        RV_PLIC.prio(i).reset().prio(1).reg.write();
        RV_PLIC.ie0(i >> 5).modify().e((i & 31) as u8, true).reg.write();
    }
}

fn machine_external() {
    loop {
        let source = RV_PLIC.cc(0).read().cc();
        match source {
            0 => break,
            plic::RV_TIMER_MIN ..= plic::RV_TIMER_MAX => crate::board::timer::interrupt(),
            plic::USBDEV_MIN ..= plic::USBDEV_MAX => crate::board::usb::interrupt(),
            _ => {
                log::error!("unexpected interrupt {}", source);
                RV_PLIC.prio(source).reset().reg.write();
                RV_PLIC.ie0(source >> 5).modify().e((source & 31) as u8, false).reg.write();
            }
        }
        RV_PLIC.cc(0).reset().cc(source).reg.write();
    }
}

#[unsafe(export_name = "DefaultHandler")]
fn default_handler() {
    match cause::<_, Exception>() {
        Trap::Interrupt(Interrupt::MachineExternal) => machine_external(),
        Trap::Interrupt(x) => log::panic!("unexpected interrupt {}", x as usize),
        Trap::Exception(x) => log::panic!("unexpected exception {}", x as usize),
    }
}
