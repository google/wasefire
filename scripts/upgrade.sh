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
. scripts/package.sh

# This script upgrades all dependencies.

for submodule in $(git submodule status | cut -d' ' -f3); do
  # The rustup script is checked in the sync.sh script.
  [ $submodule = third_party/rust-lang/rustup ] && continue
  i "Upgrade $submodule"
  ( cd $submodule
    git fetch -p origin
    head=refs/remotes/origin/HEAD
    [ $submodule = third_party/google/OpenSK ] && head=refs/remotes/origin/develop
    git checkout $head
  )
done

x sed -i 's/^\(channel = "nightly-\)[^"]*"$/\1'$(date +%F)'"/' \
  rust-toolchain.toml

get_crates() { sed -n 's/^.*ensure_cargo \([^ ]\+\) .*$/\1/p' scripts/wrapper.sh; }
update_crate() { x sed -i 's/\(ensure_cargo '"$1"'\) [0-9.]*/\1 '"$2"'/' scripts/wrapper.sh; }
for crate in $(get_crates); do
  update_crate "$crate" "$(cargo_info_version "$crate")"
done

# TODO(https://github.com/rust-lang/cargo/issues/10307): Remove the loop and inline.
update_breaking() {
  while ! x cargo -Z unstable-options update --manifest-path=$1 --breaking; do
    t 'Manually fix the issue with `cargo update <spec>` and hit ENTER'
    read garbage
  done
}
for crate in $TOPOLOGICAL_ORDER; do
  update_breaking crates/$crate/Cargo.toml
done
for path in $(git ls-files '*/Cargo.toml'); do
  update_breaking $path
done

( cd examples/assemblyscript
  x npm install --no-save assemblyscript
)
ASC_VERSION=$(sed -n 's/^  "version": "\(.*\)",$/\1/p' \
  examples/assemblyscript/node_modules/assemblyscript/package.json)
x sed -i "/ASC_VERSION:/s/\"[^\"]*\"/\"$ASC_VERSION\"/" crates/xtask/src/main.rs

x git commit -am'Upgrade all dependencies'

d "All dependencies have been upgraded and a commit created"
