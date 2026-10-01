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

# This script checks SemVer compatibility for user-facing published crates.

BASE="$1"
if [ -z "$BASE" ]; then
  for ref in upstream/main origin/main main; do
    git rev-parse --verify -q "$ref^{commit}" >/dev/null || continue
    BASE=$(git merge-base HEAD "$ref")
    break
  done
fi
[ -n "$BASE" ] || e "Could not determine the SemVer baseline"
git cat-file -e "$BASE^{commit}" || e "Invalid SemVer baseline '$BASE'"

BASELINE=$(mktemp -d)
rmdir "$BASELINE"
trap 'git worktree remove --force "$BASELINE" >/dev/null 2>&1 || true' EXIT
x git worktree add --quiet --detach "$BASELINE" "$BASE"

version() {
  sed -n '/^\[package\]$/,/^$/{s/^version = "\(.*\)"$/\1/p}' "$1"
}

for crate in board scheduler prelude; do
  manifest=crates/$crate/Cargo.toml
  current=$(version "$manifest")
  baseline=$(version "$BASELINE/$manifest")
  if [ "$current" = "$baseline" ]; then
    i "Skip $crate unchanged at $current"
    continue
  fi
  case "$crate" in
    board) features="--only-explicit-features --features=full-api,std" ;;
    scheduler) features="--only-explicit-features --features=std,wasm" ;;
    prelude) features="--default-features --features=rust-crypto" ;;
  esac
  x ./scripts/wrapper.sh cargo semver-checks check-release \
    --manifest-path="$manifest" --baseline-root="$BASELINE/crates/$crate" $features
done
