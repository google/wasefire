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

ensure_applet

test_helper

DEFMT_LOG=trace cargo check --bin=runner-opentitan --target=riscv32imc-unknown-none-elf \
--features=wasm,debug,test-vendor
cargo check --bin=runner-opentitan --target=riscv32imc-unknown-none-elf \
--features=wasm,release,ed25519
cargo check --bin=runner-opentitan --target=riscv32imc-unknown-none-elf \
--features=wasm,release,software-ed25519
cargo check --bin=runner-opentitan --target=riscv32imc-unknown-none-elf --features=wasm,release
cargo check --bin=runner-opentitan --target=riscv32imc-unknown-none-elf --features=native,debug
cargo check --bin=runner-opentitan --target=riscv32imc-unknown-none-elf --features=native,release
cargo check --bin=runner-opentitan --target=riscv32imc-unknown-none-elf \
--features=native,release,usb-ctap
