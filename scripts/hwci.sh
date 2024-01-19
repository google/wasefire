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

# This script runs the test applets and thus needs an appropriate board. It
# takes the name of the runner as argument and any runner flags, if any.

[ $# -gt 0 ] || e "Usage: $0 <runner name> [<runner flags>..]"

list() {
  find examples/rust -maxdepth 1 -name '*_test' -printf '%P\n' | sort
}

features() {
  package_features | grep -v -e human -e test
}

for name in $(list); do
  x cargo xtask applet rust $name runner "$@"
  for feature in $(cd examples/rust/$name && features); do
    native=
    if [ $feature = native ]; then
      [ "$1" = host ] && continue
      native=--native
    fi
    x cargo xtask $native applet rust $name --features=$feature runner "$@"
  done
done
for name in $(list); do
  x cargo xtask --release applet rust $name runner "$@"
done
