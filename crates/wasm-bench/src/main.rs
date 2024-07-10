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
#![cfg_attr(any(feature = "target-nordic", feature = "target-riscv"), no_main)]

extern crate alloc;
#[cfg(feature = "target-linux")]
extern crate std;

#[cfg(any(feature = "target-nordic", feature = "target-riscv"))]
mod allocator;
#[cfg_attr(feature = "runtime-base", path = "runtime/base.rs")]
#[cfg_attr(feature = "runtime-wasm3", path = "runtime/wasm3.rs")]
#[cfg_attr(feature = "runtime-wasmi", path = "runtime/wasmi.rs")]
#[cfg_attr(feature = "runtime-wasmtime", path = "runtime/wasmtime.rs")]
mod runtime;
#[cfg_attr(feature = "target-linux", path = "target/linux.rs")]
#[cfg_attr(feature = "target-nordic", path = "target/nordic.rs")]
#[cfg_attr(feature = "target-riscv", path = "target/riscv.rs")]
mod target;

// https://github.com/wasm3/wasm-coremark/blob/main/coremark-minimal.wasm
const WASM: &[u8] = include_bytes!("../../../target/wasefire/coremark-minimal.wasm");

fn main() {
    println!("Running CoreMark measurement...");
    let start = target::clock_ms();
    let result = runtime::run(WASM);
    let duration = target::clock_ms() - start;
    println!("CoreMark result: {} (in {}s)", result, duration as f32 / 1000.);
}

#[cfg(feature = "target-nordic")]
#[cortex_m_rt::entry]
fn entry() -> ! {
    target::init();
    loop {
        main();
    }
}

#[cfg(feature = "target-riscv")]
#[riscv_rt::entry]
fn entry() -> ! {
    target::init();
    loop {
        main();
    }
}
