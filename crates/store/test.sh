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

cargo check --features=std
cargo check --target=thumbv7em-none-eabi
cargo fmt -- --check
cargo clippy --features=std -- --deny=warnings
cargo clippy --target=thumbv7em-none-eabi -- --deny=warnings
cargo test --features=std
RUSTDOCFLAGS=--deny=warnings cargo doc --features=std
