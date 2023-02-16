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

# This script synchronizes generated content.

CI_WORKFLOW=.github/workflows/ci.yml
CI_SCRIPT=scripts/ci.sh
CI_CACHE_KEY="rust-\${{ hashFiles('rust-toolchain.toml', '**/Cargo.lock') }}"

ci_setup() {
  cat <<EOF
      - uses: actions/checkout@v2
      - uses: actions/cache@v3
        with:
          path: |
            ~/.rustup/
            ~/.cargo/
            target/
          key: $CI_CACHE_KEY$1
          restore-keys: $CI_CACHE_KEY
      - run: ./scripts/setup.sh
EOF
}

cat > $CI_WORKFLOW <<EOF
name: Continuous Integration

on:
  push:
    branches:
      - main
  pull_request:
    branches:
      - main
  schedule:
    - cron: 12 6 * * 2 # every Tuesday at 6:12 UTC

jobs:
  setup:
    runs-on: ubuntu-latest
    steps:
$(ci_setup)
      - run: "cd crates/xtask && cargo build"
EOF

sed -n '0,/^$/p' $0 > $CI_SCRIPT
cat >> $CI_SCRIPT <<EOF
set -ex

# This script runs the continuous integration tests.

./scripts/setup.sh
EOF

ci_step() {
  cat >> $CI_WORKFLOW <<EOF
  $1:
    needs: setup
    runs-on: ubuntu-latest
    steps:
$(ci_setup -$1)
      - run: "$2"
EOF
  echo "( $2 )" >> $CI_SCRIPT
}

ci_step applets 'cargo xtask build-applets'
ci_step applets-release 'cargo xtask --release build-applets'
ci_step runners 'cargo xtask build-runners'
ci_step runners-release 'cargo xtask --release build-runners'

for dir in $(find . -name test.sh -printf '%h\n'); do
  ci_step "$(echo ${dir#.} | tr / _ )" "cd $dir && ./test.sh"
done

ci_step sync './scripts/sync.sh && git diff --exit-code'

example_step() {
  sed '/ANCHOR/d' book/src/applet/prelude/$1.rs > examples/rust/$2/src/lib.rs
}

example_step led blink
example_step button1 button
example_step button2 led
example_step timer button_abort
example_step usb memory_game
example_step store store
