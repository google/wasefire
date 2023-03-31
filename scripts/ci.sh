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

# This script runs the continuous integration tests.

x cargo xtask build-applets
x cargo xtask --release build-applets
x cargo xtask build-runners
x cargo xtask --release build-runners

for dir in $(find . -name test.sh -printf '%h\n' | sort); do
  i "Run tests in $dir"
  ( cd $dir && ./test.sh )
done

for dir in $(find crates -name Cargo.toml -printf '%h\n' | sort); do
  sed -n '1{/^\[package\]$/!q1};/^publish =/q;/^$/q1' $dir/Cargo.toml \
    || e "Cargo.toml for $dir is missing the publish field"
  $(sed -n 's/^publish = //p;T;q' $dir/Cargo.toml) || continue
  [ -e $dir/CHANGELOG.md ] || e "CHANGELOG.md for $dir is missing"
  ( cd $dir
    ref=$(git log --first-parent -n1 --pretty=format:%H -- CHANGELOG.md)
    [ -n "$ref" ] || e "CHANGELOG.md for $dir is not tracked"
    git diff --quiet $ref.. -- $(cargo package --list) \
      || e "CHANGELOG.md for $dir is not up-to-date"
    [ "$(sed -n 's/^version = //p;T;q' Cargo.toml | tr -d \")" \
      = "$(sed -n 's/^## //p;T;q' CHANGELOG.md)" ] \
      || e "CHANGELOG.md and Cargo.toml for $dir have different versions"
  )
done

git diff --exit-code \
  || e "Tracked files were modified and/or untracked files were created"

x taplo format
git diff --exit-code || e "TOML files are not well formatted"

x ./scripts/sync.sh
git diff --exit-code || e "Generated content is not in sync"

d "All tests passed"
