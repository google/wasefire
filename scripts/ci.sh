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

for crate in $(ls crates); do
  name=${crate#runner-}
  [ $crate = $name ] && continue
  x cargo xtask runner $name --log=trace
  x cargo xtask --release runner $name
done

for dir in $(git ls-files '**/test.sh'); do
  dir=$(dirname $dir)
  i "Run tests in $dir"
  ( cd $dir && ./test.sh )
done

./scripts/hwci.sh host --no-default-features

i "Build the book"
WASEFIRE_WRAPPER_EXEC=n ./scripts/wrapper.sh mdbook
( cd book && ../scripts/wrapper.sh mdbook build 2>/dev/null )

git diff --exit-code \
  || e "Tracked files were modified and/or untracked files were created"

x ./scripts/wrapper.sh taplo lint --default-schema-catalogs
x ./scripts/wrapper.sh taplo format
git diff --exit-code || e "TOML files are not well formatted"

x ./scripts/sync.sh
git diff --exit-code || e "Generated content is not in sync"
