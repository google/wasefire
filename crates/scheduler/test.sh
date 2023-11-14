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

if [ ! -e ../../target/wasefire/libapplet.a ]; then
  ( cd ../..
    cargo xtask --native-target=thumbv7em-none-eabi applet rust hello
  )
fi
cargo check --features=wasm,std
cargo check --features=wasm,std,log
cargo check --target=i686-unknown-linux-gnu --features=native,std
cargo check --target=i686-unknown-linux-gnu --features=native,std,log
cargo check --target=thumbv7em-none-eabi --features=wasm
cargo check --target=thumbv7em-none-eabi --features=wasm,defmt
cargo check --target=thumbv7em-none-eabi --features=native
cargo check --target=thumbv7em-none-eabi --features=native,defmt
cargo fmt -- --check
sed -n '/^cargo check /!d;s/^cargo check/cargo clippy/;'\
':a;$!{N;s/\\\n//;ta};s/\n/ -- --deny=warnings\n/;P;D' $0 | sh -ex
cargo test --features=wasm,std
