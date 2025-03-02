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

x cargo xtask changelog ci

# All source files should be under /src/. In praticular, /build.rs should be under /src/build.rs and
# package.build set to point to that path.
INCLUDE='["/LICENSE", "/src/"]'
LICENSE="$(readlink -f LICENSE)"
for dir in $(find crates examples/rust -name Cargo.toml -printf '%h\n' | sort); do
  ( cd $dir
    edition="$(package_edition)"
    [ -n "$edition" ] || e "Cargo.toml for $dir should specify edition"
    [ $edition -eq 2024 ] || e "Cargo.toml for $dir should use edition 2024"
    [ ${dir#examples/rust/exercises/part-} = $dir ] || exit 0
    [ -e test.sh ] || e "test.sh for $dir is missing"
    [ ${dir#examples/rust} = $dir ] || exit 0
    publish="$(package_publish)"
    [ -n "$publish" ] || e "Cargo.toml for $dir is missing the publish field"
    if ! $publish; then
      [ "$(package_version)" = 0.1.0 ] || e "Unpublished $dir should have version 0.1.0"
      [ -e CHANGELOG.md ] && e "Unpublished $dir should not have a CHANGELOG.md"
      [ -e LICENSE ] && e "Unpublished $dir should not have a LICENSE"
      exit 0
    fi
    [ -e CHANGELOG.md ] || e "CHANGELOG.md for $dir is missing"
    [ "$(package_include)" = "$INCLUDE" ] || e "Cargo.toml for $dir should include exactly $INCLUDE"
    [ "$(readlink -f LICENSE)" = "$LICENSE" ] \
      || e "LICENSE for $dir is not a symlink to the top-level one"
    [ -z "$(package_exclude)" ] || e "Cargo.toml for $dir should not exclude anything"
    ref=$(git log -n1 --pretty=format:%H origin/main.. -- CHANGELOG.md)
    [ -n "$ref" ] || ref=origin/main
    lock=Cargo.lock
    [ -e src/main.rs ] || lock=
    git diff --quiet $ref -- Cargo.toml $lock src || e "CHANGELOG.md for $dir is not up-to-date"
    ver="$(sed -n '3s/^## //p' CHANGELOG.md)"
    [ -n "$ver" ] || e "CHANGELOG.md for $dir does not start with version"
    [ "$(package_version)" = "$ver" ] \
      || e "CHANGELOG.md and Cargo.toml for $dir have different versions"
    crate=${dir#crates/}
    if [ $dir != crates/prelude ]; then
      package_features | grep -q '^default$' && e "Cargo.toml for $dir has default features"
    fi
    sed -n '/^\[dependencies]$/,/^$/{/^wasefire-/d;/^[a-z]/!d;'\
'/default-features = false/d;p;q1};/^\[dependencies\.wasefire-/d;'\
'/^\[dependencies\./{h;:a;n;/default-features = false/d;/^$/{g;p;q1};ba}' \
Cargo.toml || e "Cargo.toml for $dir doesn't disable default-features"
  )
done
