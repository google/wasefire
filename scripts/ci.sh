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

x ./scripts/ci-copyright.sh
x ./scripts/ci-changelog.sh
x ./scripts/sync.sh
x ./scripts/ci-taplo.sh
x ./scripts/ci-applets.sh
x ./scripts/ci-runners.sh
x ./scripts/ci-tests.sh
x ./scripts/hwci.sh host --no-default-features
x ./scripts/ci-book.sh
x ./scripts/footprint.sh
x rm footprint.toml
git diff --exit-code || e 'Modified files'
[ -z "$(git status -s | tee /dev/stderr)" ] || e 'Untracked files'
d "CI passed"
