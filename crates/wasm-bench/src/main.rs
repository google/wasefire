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
#![cfg_attr(feature = "_target-embedded", no_main)]

extern crate alloc;
#[cfg(not(feature = "_target-embedded"))]
extern crate std;

#[cfg(not(feature = "_target-embedded"))]
use std::println;

#[cfg(feature = "_target-embedded")]
use semihosting::println;

#[cfg(feature = "_target-embedded")]
mod allocator;
#[cfg_attr(feature = "runtime-base", path = "runtime/base.rs")]
#[cfg_attr(feature = "runtime-wasmtime", path = "runtime/wasmtime.rs")]
mod runtime;
#[cfg_attr(feature = "target-linux", path = "target/linux.rs")]
#[cfg_attr(feature = "target-nordic", path = "target/nordic.rs")]
#[cfg_attr(feature = "target-riscv", path = "target/riscv.rs")]
mod target;

// We need to align the module for `runtime-wasmtime` because `object` expects the ELF to be
// aligned. We align it for all runtimes for simplicity.
struct Module<Bytes: ?Sized> {
    _align: [u64; 0],
    bytes: Bytes,
}
static MODULE_: &Module<[u8]> =
    &Module { _align: [], bytes: *include_bytes!(concat!(env!("OUT_DIR"), "/module.bin")) };
const MODULE: &[u8] = &MODULE_.bytes;

#[cfg_attr(feature = "target-nordic", cortex_m_rt::entry)]
#[cfg_attr(feature = "target-riscv", riscv_rt::entry)]
fn main() -> ! {
    #[cfg(feature = "_target-embedded")]
    allocator::init();
    target::init();

    println!("Running CoreMark measurement...");
    let start = target::clock_ms();
    let result = runtime::run(MODULE);
    let duration = target::clock_ms() - start;
    println!("CoreMark result: {} (in {}s)", result, duration as f32 / 1000.);

    #[cfg(feature = "_target-embedded")]
    allocator::done();
    #[cfg(feature = "_target-embedded")]
    semihosting::process::exit(0);
    #[cfg(not(feature = "_target-embedded"))]
    std::process::exit(0);
}
