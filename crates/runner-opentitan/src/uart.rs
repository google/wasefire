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

#![allow(static_mut_refs)]

use earlgrey::UART;

pub fn reset() {
    control(0);
}

pub fn exit(success: bool) -> ! {
    control(if success { 1 } else { 2 });
    crate::idle();
}

#[defmt::panic_handler]
fn defmt_panic() -> ! {
    exit(false);
}

#[defmt::global_logger]
struct Uart;

static mut CRITICAL_SECTION: Option<critical_section::RestoreState> = None;
pub static mut ENCODER: defmt::Encoder = defmt::Encoder::new();

unsafe impl defmt::Logger for Uart {
    fn acquire() {
        // SAFETY: Will call a properly-nested release.
        let restore_state = unsafe { critical_section::acquire() };

        // SAFETY: We are in a critical section.
        let critical_section = unsafe { &mut CRITICAL_SECTION };

        if critical_section.is_some() {
            // SAFETY: Matches the acquire above.
            unsafe { critical_section::release(restore_state) };
            panic!("Logger already acquired.");
        }
        *critical_section = Some(restore_state);

        // SAFETY: We are in a critical section.
        let encoder = unsafe { &mut ENCODER };
        encoder.start_frame(write);
        write(&[3]); // control code for xtask to know it's a defmt frame
    }

    unsafe fn flush() {}

    unsafe fn release() {
        // SAFETY: We are in a critical section.
        let restore_state = unsafe { &mut CRITICAL_SECTION }.take().unwrap();

        // SAFETY: We are in a critical section.
        let encoder = unsafe { &mut ENCODER };
        encoder.end_frame(write);

        // SAFETY: Matches the acquire in Self::acquire.
        unsafe { critical_section::release(restore_state) };
    }

    unsafe fn write(bytes: &[u8]) {
        // SAFETY: We are in a critical section.
        let encoder = unsafe { &mut ENCODER };
        encoder.write(bytes, write);
    }
}

fn control(code: u8) {
    // We make sure the previously written byte is a null byte. This relies on the
    // Encoder::start_frame() implementation.
    critical_section::with(|_| {
        // SAFETY: We are in a critical section.
        unsafe { &mut ENCODER }.start_frame(write);
    });
    write(&[code]);
}

fn write(bytes: &[u8]) {
    for &byte in bytes {
        putchar(byte);
    }
}

fn putchar(byte: u8) {
    while UART[0].status().read().txfull() {}
    UART[0].wdata().write_raw(byte as u32);
    while !UART[0].status().read().txidle() {}
}
