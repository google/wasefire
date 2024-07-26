#!/bin/sh
# Copyright 2024 Google LLC
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

set -e

. "$(git rev-parse --show-toplevel)"/scripts/test-helper.sh

ensure_submodule third_party/wasm3/wasm-coremark

test_helper

cargo test --bin=wasm-bench --features=target-linux,runtime-base
cargo test --bin=wasm-bench --features=target-linux,runtime-wasm3
cargo test --bin=wasm-bench --features=target-linux,runtime-wasmi
cargo test --bin=wasm-bench --features=target-linux,runtime-wasmtime

cargo check --bin=wasm-bench --target=thumbv7em-none-eabi --features=target-nordic,runtime-base
# wasm3/source/wasm3.h:16:10: fatal error: 'stdlib.h' file not found
# cargo check --bin=wasm-bench --target=thumbv7em-none-eabi --features=target-nordic,runtime-wasm3
cargo check --bin=wasm-bench --target=thumbv7em-none-eabi --features=target-nordic,runtime-wasmi
# error in crate arbitrary: can't find crate for `std`
# cargo check --bin=wasm-bench --target=thumbv7em-none-eabi --features=target-nordic,runtime-wasmtime

cargo check --bin=wasm-bench --target=riscv32imc-unknown-none-elf \
  --features=target-riscv,runtime-base
# error: unknown target triple 'riscv32imc-unknown-none-elf', please use -triple or -arch
# cargo check --bin=wasm-bench --target=riscv32imc-unknown-none-elf \
#   --features=target-riscv,runtime-wasm3
# error: no method named `compare_exchange` found for struct `AtomicUsize` in the current scope
# cargo check --bin=wasm-bench --target=riscv32imc-unknown-none-elf \
#   --features=target-riscv,runtime-wasmi
# error in crate once_cell: can't find crate for `std`
# cargo check --bin=wasm-bench --target=riscv32imc-unknown-none-elf \
#   --features=target-riscv,runtime-wasmtime
