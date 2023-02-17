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

# External tests.
cargo xtask build-applets
cargo xtask --release build-applets
cargo xtask build-runners
cargo xtask --release build-runners

# Internal tests.
for dir in $(find . -name test.sh -printf '%h\n' | sort); do
  ( cd $dir && ./test.sh )
done

# Make sure generated content is synced.
git diff --exit-code
./scripts/sync.sh
git diff --exit-code
