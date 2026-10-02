#!/bin/sh
# Copyright 2026 Google LLC
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

# This script checks SemVer compatibility for user-facing published crates.

BASE="$(git tag -l 'release/*' | tail -n1)"
[ -n "$BASE" ] || e "Failed to find latest release"

BASELINE="$(mktemp -du)"
trap 'git worktree remove --force "$BASELINE" >/dev/null 2>&1 || true' EXIT
x git worktree add --quiet --detach "$BASELINE" "$BASE"

check() {
  local dir=$1; shift
  local ver=$(cd $dir && package_version)
  if [ "$ver" = "$(cd "$BASELINE/$dir" && package_version)" ]
  then i "Skip $dir unchanged at $ver"
  else x ./scripts/wrapper.sh cargo semver-checks \
         --manifest-path=$dir --baseline-root="$BASELINE/$dir" "$@"
  fi
}

# TODO(https://github.com/obi1kenobi/cargo-semver-checks/issues/1746): Uncomment when fixed.
# check crates/board --only-explicit-features --features=full-api,std
# check crates/scheduler --only-explicit-features --features=std,wasm
check crates/prelude --default-features --features=rust-crypto
