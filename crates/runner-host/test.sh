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

set -ex

if [ ! -e ../../target/wasefire/applet.wasm ]; then
  mkdir -p ../../target/wasefire
  touch ../../target/wasefire/applet.wasm
fi
if [ ! -e ../../target/wasefire/libapplet.a ]; then
  ( cd ../..
    cargo xtask --native-target=thumbv7em-none-eabi applet rust hello
  )
fi
cargo check --features=wasm,debug
cargo check --features=wasm,debug,web
cargo check --features=wasm,release
cargo check --target=i686-unknown-linux-gnu --features=native,release
cargo check --no-default-features --features=wasm,debug
cargo fmt -- --check
cargo clippy --features=wasm,debug -- --deny=warnings
cargo clippy --features=wasm,debug,web -- --deny=warnings
cargo clippy --features=wasm,release -- --deny=warnings
cargo clippy --target=i686-unknown-linux-gnu --features=native,release -- \
--deny=warnings
cargo clippy --no-default-features --features=wasm,debug -- --deny=warnings
cargo test --features=wasm,debug
