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

for lang in $(ls examples); do
  for name in $(ls examples/$lang); do
    [ $lang = assemblyscript -a $name = node_modules ] && continue
    [ $lang = assemblyscript -a $name = api.ts ] && continue
    x cargo xtask applet $lang $name
    x cargo xtask --release applet $lang $name
  done
done

for crate in $(ls crates); do
  name=${crate#runner-}
  [ $crate = $name ] && continue
  x cargo xtask runner $name --log=trace
  x cargo xtask --release runner $name
done

for dir in $(find . -name test.sh -printf '%h\n' | sort); do
  i "Run tests in $dir"
  ( cd $dir && ./test.sh )
done

./scripts/hwci.sh host --no-default-features

i "Build the book"
WASEFIRE_WRAPPER_EXEC=n ./scripts/wrapper.sh mdbook
( cd book && ../scripts/wrapper.sh mdbook build 2>/dev/null )

./scripts/ci-changelog.sh

git diff --exit-code \
  || e "Tracked files were modified and/or untracked files were created"

x ./scripts/wrapper.sh taplo format
git diff --exit-code || e "TOML files are not well formatted"

x ./scripts/sync.sh
git diff --exit-code || e "Generated content is not in sync"
