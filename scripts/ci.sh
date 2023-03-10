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

set -ex

# This script runs the continuous integration tests.

# External tests are passing.
cargo xtask build-applets
cargo xtask --release build-applets
cargo xtask build-runners
cargo xtask --release build-runners

# Internal tests are passing.
for dir in $(find . -name test.sh -printf '%h\n' | sort); do
  ( cd $dir && ./test.sh )
done

# All crates specify whether they are published.
for dir in $(find . -name Cargo.toml -printf '%h\n' | sort); do
  sed -n '1{/^\[package\]$/!q1};/^publish =/q;/^$/q1' $dir/Cargo.toml

  # All published crates have an up-to-date CHANGELOG.md.
  $(sed -n 's/^publish = //p;T;q' $dir/Cargo.toml) || continue
  [ -e $dir/CHANGELOG.md ]
  ( cd $dir
    ref=$(git log --first-parent -n1 --pretty=format:%H -- CHANGELOG.md)
    [ -n "$ref" ]
    git diff --quiet $ref.. -- $(cargo package --list)
    [ "$(sed -n 's/^version = //p;T;q' Cargo.toml | tr -d \")" \
      = "$(sed -n 's/^## //p;T;q' CHANGELOG.md)" ]
  )
done

# No tracked files were modified and no untracked files were created.
git diff --exit-code

# TOML files are well-formatted.
taplo format
git diff --exit-code

# Generated content is in sync.
./scripts/sync.sh
git diff --exit-code
