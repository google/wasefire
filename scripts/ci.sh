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

# This script runs the continuous integration tests.

./scripts/ci-copyright.sh
./scripts/ci-changelog.sh
./scripts/sync.sh
./scripts/ci-taplo.sh
./scripts/ci-applets.sh
./scripts/ci-runners.sh
./scripts/ci-tests.sh
./scripts/hwci.sh host --no-default-features
./scripts/ci-book.sh
./scripts/footprint.sh
rm footprint.toml
git diff --exit-code
