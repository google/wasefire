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

test_helper

cargo test --bin=runner-host --features=wasm,debug
cargo check --bin=runner-host --features=wasm,debug,web
cargo check --bin=runner-host --features=wasm,release
# TODO(https://github.com/a1ien/rusb/issues/211): Uncomment when fixed.
# cargo check --bin=runner-host --target=i686-unknown-linux-gnu --features=native,release
cargo check --bin=runner-host --no-default-features --features=wasm,debug,tcp
cargo check --bin=runner-host --no-default-features --features=wasm,debug,unix
