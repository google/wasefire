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

cargo test --bin=wasm-bench --target=i686-unknown-linux-gnu --features=target-linux,runtime-base
cargo test --bin=wasm-bench --target=i686-unknown-linux-gnu --features=target-linux,runtime-wasmtime

cargo check --bin=wasm-bench --target=thumbv7em-none-eabi --features=target-nordic,runtime-base
cargo check --bin=wasm-bench --target=thumbv7em-none-eabi --features=target-nordic,runtime-wasmtime

cargo check --bin=wasm-bench --target=riscv32imc-unknown-none-elf \
  --features=target-riscv,runtime-base
# error: unresolved import `alloc::sync`
# cargo check --bin=wasm-bench --target=riscv32imc-unknown-none-elf \
#   --features=target-riscv,runtime-wasmtime
