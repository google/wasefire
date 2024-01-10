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

check_software_crypto
check_applet_api applet-api- --features=wasm,std, --all-targets
check_board_api board-api- --features=wasm,std, --all-targets

test_helper

cargo test --features=full-api,wasm,std
cargo check --features=full-api,wasm,std,log
cargo check --target=i686-unknown-linux-gnu --features=full-api,native,std
cargo check --target=i686-unknown-linux-gnu \
  --features=full-api,native,std,log
cargo check --target=thumbv7em-none-eabi --features=full-api,wasm
cargo check --target=thumbv7em-none-eabi --features=full-api,wasm,defmt
cargo check --target=thumbv7em-none-eabi --features=full-api,native
cargo check --target=thumbv7em-none-eabi --features=full-api,native,defmt
