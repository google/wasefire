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

set -ex

# This is a generic test script. We don't need the test-helper script.

for part in part-*; do
  ( cd $part
    # This is only to update the Cargo.lock file.
    RUSTFLAGS=--allow=warnings cargo check --target=wasm32-unknown-unknown
    # We check the usual for solutions though.
    [ ${part%-sol} = $part ] && exit
    cargo fmt -- --check
    cargo clippy --target=wasm32-unknown-unknown -- --deny=warnings
  )
done
