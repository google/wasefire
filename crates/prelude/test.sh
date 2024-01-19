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

check_applet_api api- --features=rust-crypto, --target=wasm32-unknown-unknown \
  --no-default-features

for api in api-rng api-crypto-hash api-crypto-hkdf; do
  cargo clippy --target=wasm32-unknown-unknown --no-default-features \
    --features=api-crypto-ec,$api -- --deny=warnings
done

test_helper

cargo check --target=wasm32-unknown-unknown --features=full-api
cargo check --target=wasm32-unknown-unknown --features=full-api,rust-crypto
cargo check --features=full-api,native
cargo test --features=full-api,test
