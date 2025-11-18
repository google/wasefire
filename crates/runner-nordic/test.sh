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

ensure_applet

for feature in $(package_features); do
  case $feature in _*|board-*|debug|release|native|pulley|wasm) continue ;; esac
  x cargo clippy --bin=runner-nordic --target=thumbv7em-none-eabi \
--features=wasm,debug,board-devkit,$feature -- --deny=warnings
done

test_helper

cargo check --bin=runner-nordic --target=thumbv7em-none-eabi \
--features=wasm,debug,board-devkit,test-vendor,_full
cargo check --bin=runner-nordic --target=thumbv7em-none-eabi --features=wasm,debug,board-devkit
DEFMT_LOG=trace cargo check --bin=runner-nordic --target=thumbv7em-none-eabi \
--features=wasm,debug,board-devkit
cargo check --bin=runner-nordic --target=thumbv7em-none-eabi --features=pulley,debug,board-devkit
cargo check --bin=runner-nordic --target=thumbv7em-none-eabi --features=wasm,release,board-dongle
cargo check --bin=runner-nordic --target=thumbv7em-none-eabi --features=native,release,board-dongle
cargo check --bin=runner-nordic --target=thumbv7em-none-eabi \
--features=native,release,board-makerdiary
