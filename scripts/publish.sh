#!/bin/sh
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
. scripts/test-helper.sh

# This script publishes all crates.

diff_sorted TOPOLOGICAL_ORDER \
  "$(git ls-files '*/Cargo.toml' | sed -n 's#^crates/\(.*\)/Cargo.toml$#\1#p' | sort)" \
  $(echo $TOPOLOGICAL_ORDER | sed 's/ /\n/g' | sort)

dependencies() {
  sed -n 's#^.*path = "\([^"]*\)".*$#\1#p' crates/$1/Cargo.toml | \
    while read i; do
      [ ${i%.rs} = $i ] || continue
      realpath --relative-base=crates -m crates/$1/$i
    done
}

occurs_before() {
  for x in $TOPOLOGICAL_ORDER; do
    [ $x = $1 ] && return
    [ $x = $2 ] && return 1
  done
  return 2
}

for crate in $TOPOLOGICAL_ORDER; do
  for dep in $(dependencies $crate); do
    occurs_before $dep $crate || e "$crate depends on $dep but occurs before"
  done
done

[ "$1" = --dry-run ] && d "Nothing more to do with --dry-run"

[ -z "$(git status -s)" ] || e "Repository is not clean"

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

for crate in $TOPOLOGICAL_ORDER; do
  ( cd crates/$crate
    $(package_publish) || exit 0
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

d "You can now run ./scripts/announce.sh"
