#!/bin/sh
# Copyright 2022 Google LLC
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

ensure_submodule third_party/WebAssembly/spec

list_files() {
  find ../../third_party/WebAssembly/spec/test/core \
    -maxdepth 1 -name '*.wast' -execdir basename -s .wast {} \;
}
list_tests() {
  sed -n 's/^test!(.*, "\([^"]*\)".*);$/\1/p;s/^test!(\([^,]*\).*);$/\1/p' tests/spec.rs
}
diff_sorted tests/spec.rs "$(list_files | sort)" $(list_tests)

test_helper

cargo test --lib --features=debug,toctou
cargo check --lib --target=thumbv7em-none-eabi
cargo check --lib --target=riscv32imc-unknown-none-elf \
  --features=portable-atomic/critical-section
RUSTFLAGS=--cfg=portable_atomic_unsafe_assume_single_core \
  cargo check --lib --target=riscv32imc-unknown-none-elf
cargo check --example=hello
# Run with `-- --test-threads=1 --nocapture` to see unsupported tests.
cargo test --test=spec --features=debug,toctou,float-types,vector-types
