#!/bin/sh
# Copyright 2023 Google LLC
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

# This script upgrades all dependencies.

x sed -i 's/^\(channel = "nightly-\)[^"]*"$/\1'$(date +%F)'"/' \
  rust-toolchain.toml
for submodule in WebAssembly/spec; do
  i "Upgrade $submodule"
  ( cd third_party/$submodule
    git fetch -p origin
    git checkout origin/main
  )
done
for path in $(git ls-files '**/Cargo.toml'); do
  ./scripts/wrapper.sh cargo upgrade --manifest-path=$path --incompatible
done
for path in $(git ls-files '**/Cargo.toml'); do
  cargo update --manifest-path=$path
done

get_crates() {
  sed -n 's/^.*ensure_cargo \([^ ]\+\) .*$/\1/p' scripts/wrapper.sh
}

get_latest() {
  cargo search "$1" | sed -n '1s/^'"$1"' = "\([0-9.]*\)".*$/\1/p'
}

update_crate() {
  x sed -i 's/\(ensure_cargo '"$1"'\) [0-9.]*/\1 '"$2"'/' scripts/wrapper.sh
}

for crate in $(get_crates); do
  update_crate "$crate" "$(get_latest "$crate")"
done

( set -x
  cd examples/assemblyscript
  npm install --no-save assemblyscript
)
ASC_VERSION=$(sed -n 's/^  "version": "\(.*\)",$/\1/p' \
  examples/assemblyscript/node_modules/assemblyscript/package.json)
x sed -i "/ASC_VERSION:/s/\"[^\"]*\"/\"$ASC_VERSION\"/" crates/xtask/src/main.rs

d "All dependencies have been upgraded"
