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
x git submodule foreach 'git fetch -p origin && git checkout origin/main'
x find . -name Cargo.toml -print -execdir cargo upgrade --incompatible \;

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
