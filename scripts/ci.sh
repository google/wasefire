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
. scripts/log.sh
. scripts/package.sh

# This script runs the continuous integration tests.

x cargo xtask applet rust opensk
x cargo xtask --release applet rust opensk
( cd examples/rust/opensk
  x cargo fmt -- --check
  # TODO: Enable this back at some point.
  # x cargo clippy --lib --target=wasm32-unknown-unknown -- --deny=warnings
  if package_features | grep -q '^test$'; then
    # TODO: Enable this back at some point.
    # x cargo clippy --features=test -- --deny=warnings
    grep -q '^mod tests {$' src/lib.rs && x cargo test --features=test
  fi
)

git diff --exit-code \
  || e "Tracked files were modified and/or untracked files were created"

x ./scripts/schemastore.sh
x ./scripts/wrapper.sh taplo lint \
--schema-catalog=file://"$PWD"/target/schemastore/catalog.json
x ./scripts/wrapper.sh taplo format
git diff --exit-code || e "TOML files are not well formatted"
