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

# This script runs the continuous integration tests for applets.

for lang in $(ls examples); do
  for name in $(ls examples/$lang); do
    [ $lang = assemblyscript -a $name = node_modules ] && continue
    [ $lang = assemblyscript -a $name = api.ts ] && continue
    [ $lang = rust -a $name = exercises ] && continue
    x cargo xtask applet $lang $name
    x cargo xtask --release applet $lang $name
    [ $lang = rust ] || continue
    i "Run lints and tests for applet $name"
    ( cd examples/rust/$name
      x cargo fmt -- --check
      x cargo clippy --lib --target=wasm32-unknown-unknown -- --deny=warnings
      if package_features | grep -q '^test$'; then
        x cargo clippy --features=test -- --deny=warnings
        grep -q '^mod tests {$' src/lib.rs && x cargo test --features=test
        [ -e src/main.rs ] && x env WASEFIRE_DEBUG=1 cargo run --features=test
      fi
    )
  done
done
