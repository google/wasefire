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

bool() { if [ -n "$1" ]; then echo "$2"; else echo "$3"; fi; }

runner=nordic
for applet in hello hsm; do
  for release in '' --release; do
    for native in '' --native-target=thumbv7em-none-eabi; do
      config="$(bool "$release" release debug)"
      config="$config $(bool "$native" native wasm)"
      x cargo xtask --footprint="applet $applet $config" \
        $release $native applet rust $applet
      x cargo xtask --footprint="runner $runner $config" \
        $release ${native:+--native} runner $runner
    done
  done
done
