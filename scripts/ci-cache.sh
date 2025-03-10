#!/bin/sh
# Copyright 2025 Google LLC
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

# This script builds the CI cache.

./scripts/setup.sh

x cargo build --manifest-path=crates/xtask/Cargo.toml

build() {
  WASEFIRE_WRAPPER_EXEC=n ./scripts/wrapper.sh "$@"
}

CASE='\([a-z-]*\)[|)]'
PARENT="s/^  $CASE\$/=\\1/p"
CHILD="s/^      $CASE.*\$/.\\1/p"
NORMAL="s/^  $CASE.*\$/\\1/p"
LAST=
for cmd in $(sed -n "$PARENT;$CHILD;$NORMAL" scripts/wrapper.sh); do
  case "$cmd" in
    =*) LAST=${cmd#=}; continue ;;
    .*) build $LAST ${cmd#.} ;;
    *) build $cmd ;;
  esac
done
