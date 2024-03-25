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

# This script checks that Cargo.toml and CHANGELOG.md files are correct.

for dir in $(find crates -name Cargo.toml -printf '%h\n' | sort); do
  ( cd $dir
    publish="$(package_publish)"
    [ -n "$publish" ] || e "Cargo.toml for $dir is missing the publish field"
    $publish || exit 0
    [ -e CHANGELOG.md ] || e "CHANGELOG.md for $dir is missing"
    [ "$(package_include)" = '["/src"]' ] \
      || e "Cargo.toml should only include the src directory"
    [ -z "$(package_exclude)" ] || e "Cargo.toml should not exclude anything"
    ref=$(git log -n1 --pretty=format:%H origin/main.. -- CHANGELOG.md)
    [ -n "$ref" ] || ref=origin/main
    git diff --quiet $ref -- Cargo.toml src \
      || e "CHANGELOG.md for $dir is not up-to-date"
    ver="$(sed -n '3s/^## //p' CHANGELOG.md)"
    [ -n "$ver" ] || e "CHANGELOG.md for $dir does not start with version"
    [ "$(package_version)" = "$ver" ] \
      || e "CHANGELOG.md and Cargo.toml for $dir have different versions"
    if [ $dir != crates/prelude ]; then
      package_features | grep -q '^default$' \
        && e "Cargo.toml for $dir has default features"
    fi
    case $dir in crates/cli|crates/cli-tools) exit 0 ;; esac
    sed -n '/^\[dependencies]$/,/^$/{/^wasefire-/d;/^[a-z]/!d;'\
'/default-features = false/d;p;q1};/^\[dependencies\.wasefire-/d;'\
'/^\[dependencies\./{h;:a;n;/default-features = false/d;/^$/{g;p;q1};ba}' \
Cargo.toml || e "Cargo.toml for $dir doesn't disable default-features"
  )
done
