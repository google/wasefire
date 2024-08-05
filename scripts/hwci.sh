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

# This script runs the test applets and thus needs a connected platform.
#
# Usage: {host,nordic}
#   Runs all supported hardware tests for an xtask runner.
# Usage: [<protocol> [--release]]
#   Runs simple hardware tests for generic platforms.

list() {
  find examples/rust -maxdepth 1 -name '*_test' -printf '%P\n' | sort
}

features() {
  package_features | grep -v -e human -e test
}

# <protocol> {,--release} [<runner..>]
run() {
  local protocol=$1 release=$2
  local name feature runner
  shift 2
  for name in $(list); do
    [ $# -gt 0 ] || x cargo xtask $release applet rust $name install $protocol wait
    for feature in $(cd examples/rust/$name && features); do
      if [ $feature = native ]; then
        [ $# -gt 0 ] || continue
        y cargo xtask $release --native \
          applet rust $name --features=native \
          runner "$@" flash --reset-flash
        runner=$!
        x cargo xtask wait-applet $protocol
        x cargo wasefire platform-lock $protocol
        x kill $runner
      else
        [ $# -gt 0 ] && continue
        x cargo xtask $release applet rust $name --features=$feature install $protocol wait
      fi
    done
  done
}

# <protocol> <runner..>
full() {
  local protocol=--protocol=$1
  local release
  shift
  trap "trap 'exit 1' TERM && kill -- -$$" EXIT
  cargo wasefire platform-lock $protocol 2>/dev/null || true
  for release in '' --release; do
    y cargo xtask $release runner "$@" flash --reset-flash
    runner=$!
    x cargo xtask wait-platform $protocol
    run $protocol "$release"
    x cargo wasefire platform-lock $protocol
    x kill $runner
    [ $1 = host ] || run $protocol "$release" "$@"
  done
  trap - EXIT
}

case $1 in
  host) full unix host --no-default-features --features=unix ;;
  # P1.01, P1.02, and P1.03 must be connected together (gpio_test).
  nordic) full usb nordic ;;
  *) run "$@" ;;
esac
