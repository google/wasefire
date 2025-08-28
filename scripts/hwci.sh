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
# Usage: {host,nordic,opentitan}
#   Runs all supported hardware tests for an xtask runner.
# Usage: [<protocol> [--release]]
#   Runs simple hardware tests for generic platforms. An xtask runner can
#   be simulated with RUNNER={host,nordic,opentitan} in the environment.

list() {
  find examples/rust -maxdepth 1 -name '*_test' -printf '%P\n' | sort
}

features() {
  package_features | grep -v -e human -e test
}

# <protocol> {,--release} [<runner..>]
run() {
  local protocol=$1 release=$2
  local name name_flags feature features
  local runner_specific runner_feature runner_default
  shift 2
  for name in $(list); do
    runner_specific=n
    runner_feature=
    runner_default=n
    features=
    for feature in $(cd examples/rust/$name && features); do
      case $feature in
        runner-*) runner_specific=y ;;
        *) features="$features $feature" ;;
      esac
      [ $feature = runner-$RUNNER ] && runner_feature=--features=$feature
      [ $feature = runner- ] && runner_default=y
    done
    [ $runner_default = y -a -z "$runner_feature" ] && runner_feature=--features=runner-
    [ $runner_specific = y -a -z "$runner_feature" ] && continue
    name_flags="$name $runner_feature"
    [ $# -gt 0 ] || x cargo xtask $release applet rust $name_flags install $protocol wait
    for feature in $features; do
      name_flags="$name $runner_feature --features=$feature"
      if [ $feature = native ]; then
        [ $# -gt 0 ] || continue
        x cargo xtask $release --native applet rust $name_flags runner "$@" update $protocol
        x cargo xtask wait-applet $protocol
      else
        [ $# -gt 0 ] && continue
        x cargo xtask $release applet rust $name_flags install $protocol wait
      fi
    done
  done
}

# <protocol> <runner..>
full() {
  local protocol=--protocol=$1
  local release pid
  shift
  RUNNER=$1
  trap "trap 'exit 1' TERM && kill -- -$$" EXIT
  cargo wasefire platform-lock --timeout=200ms $protocol 2>/dev/null || true
  for release in '' --release; do
    y cargo xtask --setsid $release runner "$@" flash --reset-flash $FLASH_ARGS
    pid=$!
    x cargo xtask wait-platform $protocol
    run $protocol "$release"
    run $protocol "$release" "$@"
    x cargo wasefire platform-lock $protocol
    x kill -TERM -$pid
    sleep 1 # for the OS to cleanup probe-rs resources (claimed USB interface)
  done
  trap - EXIT
}

case $1 in
  host) FLASH_ARGS=--protocol=unix full unix host ;;
  # P1.01, P1.02, and P1.03 must be connected together (gpio_test).
  nordic) full usb nordic --features=test-vendor ;;
  opentitan) full usb opentitan --features=test-vendor ;;
  *) run --protocol=${1:-usb} "$2" ;;
esac
