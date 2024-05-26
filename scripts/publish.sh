#!/bin/bash
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

# This script publishes all crates.

[ -z "$(git status -s)" ] || e "Repository is not clean"

TOPOLOGICAL_ORDER=(
  logger
  error
  sync
  protocol
  cli-tools
  cli
  interpreter
  store
  api-desc
  api-macro
  api
  stub
  prelude
  board
  scheduler
  protocol-usb
)

listed_crates() {
  echo "${TOPOLOGICAL_ORDER[@]}" | sed 's/ /\n/g' | sort
}

published_crates() {
  find crates -name CHANGELOG.md -printf '%h\n' | sed 's#^crates/##' | sort
}

dependencies() {
  sed -n 's#^wasefire-.*path = "../\([a-z-]*\)".*$#\1#p' crates/$1/Cargo.toml
}

occurs_before() {
  for x in "${TOPOLOGICAL_ORDER[@]}"; do
    [ $x = $1 ] && return
    [ $x = $2 ] && return 1
  done
  return 2
}

diff <(listed_crates) <(published_crates) \
  || e 'Listed and published crates do not match (see diff above)'

for crate in "${TOPOLOGICAL_ORDER[@]}"; do
  for name in $(dependencies $crate); do
    occurs_before $name $crate || e "$crate depends on $name but occurs before"
  done
done

[ "$1" = --no-dry-run ] || d "Run with --no-dry-run from the merged PR to actually publish"

i "Remove all -git suffixes"
sed -i 's/-git//' $(git ls-files '*/'{Cargo.{toml,lock},CHANGELOG.md})
if [ -n "$(git status -s)" ]; then
  i "Commit release"
  git commit -aqm'Release all crates'
fi

get_latest() {
  name="$(package_name)"
  cargo search "$name" | sed -n '1s/^'"$name"' = "\([0-9.]*\)".*$/\1/p'
}

for crate in "${TOPOLOGICAL_ORDER[@]}"; do
  ( cd crates/$crate
    current="$(package_version)"
    latest="$(get_latest)"
    if [ "$current" = "$latest" ]; then
      i "Skipping $crate already published at $latest"
      exit
    fi
    i "Publish $crate from $latest to $current"
    eval "$(sed -n 's/^cargo \(check\|test\) --\(lib|bin=[^ ]*\)/cargo publish/p;T;q' test.sh)"
  )
done
