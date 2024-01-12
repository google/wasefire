#!/bin/sh
# Copyright 2024 Google LLC
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

# This script computes the footprint of different applets and runners in
# different configurations. The continuous integration will use this to show
# footprint impact in pull requests.

[ -e footprint.toml ] && e "footprint.toml already exists"

APPLET=exercises/part-7-sol
RUNNER=nordic

measure() {
  local release
  local native
  local opt_level
  [ $1 = release ] && release=--release
  [ $2 = native ] && native=--native-target=thumbv7em-none-eabi
  [ $3 = small ] && opt_level=--opt-level=z
  x cargo xtask --footprint="$*" $release $native applet rust $APPLET $opt_level
  [ $2 = native ] && native=--native
  x cargo xtask --footprint="$*" $release $native runner $RUNNER $opt_level
}

measure debug wasm fast
measure release wasm fast
measure release native fast
measure release native small
