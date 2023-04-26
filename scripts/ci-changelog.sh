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

# This script checks that CHANGELOG.md files are correct.

for dir in $(find crates -name Cargo.toml -printf '%h\n' | sort); do
  sed -n '1{/^\[package\]$/!q1};/^publish =/q;/^$/q1' $dir/Cargo.toml \
    || e "Cargo.toml for $dir is missing the publish field"
  $(sed -n 's/^publish = //p;T;q' $dir/Cargo.toml) || continue
  [ -e $dir/CHANGELOG.md ] || e "CHANGELOG.md for $dir is missing"
  ( cd $dir
    ref=$(git log -n1 --pretty=format:%H origin/main.. -- CHANGELOG.md)
    [ -n "$ref" ] || ref=origin/main
    git diff --quiet $ref.. -- $(cargo package --list) \
      || e "CHANGELOG.md for $dir is not up-to-date"
    ver="$(sed -n '3s/^## //p' CHANGELOG.md)"
    [ -n "$ver" ] || e "CHANGELOG.md for $dir does not start with version"
    [ "$(sed -n 's/^version = //p;T;q' Cargo.toml | tr -d \")" = "$ver" ] \
      || e "CHANGELOG.md and Cargo.toml for $dir have different versions"
  )
done
