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

./scripts/setup.sh
( cargo xtask build-applets )
( cargo xtask --release build-applets )
( cargo xtask build-runners )
( cargo xtask --release build-runners )
( cd ./crates/api && ./test.sh )
( cd ./crates/api-desc && ./test.sh )
( cd ./crates/board && ./test.sh )
( cd ./crates/interpreter && ./test.sh )
( cd ./crates/logger && ./test.sh )
( cd ./crates/prelude && ./test.sh )
( cd ./crates/runner-host && ./test.sh )
( cd ./crates/runner-nordic && ./test.sh )
( cd ./crates/scheduler && ./test.sh )
( cd ./crates/store && ./test.sh )
( cd ./crates/store/fuzz && ./test.sh )
( cd ./crates/xtask && ./test.sh )
( cd ./examples/rust/hsm/common && ./test.sh )
( cd ./examples/rust/hsm/host && ./test.sh )
( ./scripts/sync.sh && git diff --exit-code )
