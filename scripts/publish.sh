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
  one-of
  logger
  wire-derive
  error
  wire
  sync
  protocol
  interpreter
  store
  api-desc
  api-macro
  api
  stub
  prelude
  board
  scheduler
  protocol-tokio
  protocol-usb
  cli-tools
  cli
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

[ "$1" = --dry-run ] && d "Nothing more to do with --dry-run"

i "Remove all -git suffixes (if any) and reset CHANGELOG tests"
sed -i 's/-git//' $(git ls-files '*/'{Cargo.{toml,lock},CHANGELOG.md})
sed -i 's/\(^<!-- .* test\): [0-9]* -->$/\1: 0 -->/' $(git ls-files '*/CHANGELOG.md')
if [ -n "$(git status -s)" ]; then
  i "Commit release"
  git commit -aqm'Release all crates'
  t "Create a PR from this release commit"
  d "Then re-run from the merged PR"
fi

i "Final checks before actually publishing"
git log -1 --pretty=%s | grep -q '^Release all crates (#[0-9]*)$' \
  || e "This is not a merged release commit"
[ "$1" = --no-dry-run ] || d "Run with --no-dry-run to actually publish"

for crate in "${TOPOLOGICAL_ORDER[@]}"; do
  ( cd crates/$crate
    current="$(package_version)"
    latest="$(cargo_info_version "$(package_name)")"
    if [ "$current" = "$latest" ]; then
      i "Skipping $crate already published at $latest"
      exit
    fi
    i "Publish $crate from ${latest:--} to $current"
    eval "$(sed -En 's/^cargo (check|test) --(lib|bin=[^ ]*)/cargo publish/p;T;q' test.sh)"
  )
done
