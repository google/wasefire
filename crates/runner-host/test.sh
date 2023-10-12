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
cargo check --features=debug
cargo check --features=release
cargo check --no-default-features --features=debug
cargo fmt -- --check
cargo clippy --features=debug -- --deny=warnings
cargo test --features=debug
