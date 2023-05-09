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

# This script publishes all crates.

[ -z "$(git status -s)" ] || e "Repository is not clean"

TOPOLOGICAL_ORDER=(
  logger
  cli
  interpreter
  store
  api-desc
  api-macro
  api
  prelude
  board
  scheduler
)

listed_crates() {
  echo "${TOPOLOGICAL_ORDER[@]}" | sed 's/ /\n/g' | sort
}

published_crates() {
  find crates -name CHANGELOG.md -printf '%h\n' | sed 's#^crates/##' | sort
}

diff <(listed_crates) <(published_crates) \
  || e 'Listed and published crates do not match (see diff above)'

i "Remove all -git suffixes"
find . \( -name Cargo.toml -or -name Cargo.lock -or -name CHANGELOG.md \) \
  -exec sed -i 's/-git//' {} \;
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
    # TODO(#132): Remove this corner case.
    if [ $crate = scheduler ]; then
      cargo publish --no-verify
    else
      eval "$(sed -n 's/^cargo check/cargo publish/p;T;q' test.sh)"
    fi
  )
done
