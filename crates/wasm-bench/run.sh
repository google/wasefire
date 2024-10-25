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

case "$1" in
  linux) ;;
  nordic) TARGET=--target=thumbv7em-none-eabi ;;
  riscv) TARGET=--target=riscv32imc-unknown-none-elf ;;
  *) e "Unsupported target: $1" ;;
esac

case "$2" in
  base|wasm3|wasmi|wasmtime) ;;
  *) e "Unsupported runtime: $2" ;;
esac

# See test.sh for supported (and tested) combinations.
case $1-$2 in
  *-base|nordic-wasmi) ;;
  *) e "Unsupported combination: $1 $2" ;;
esac

FEATURES=--features=target-$1,runtime-$2
shift 2

x cargo run --release $TARGET $FEATURES "$@"
