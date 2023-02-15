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

if [ ! -e third_party/WebAssembly/spec/.git ]; then
  ( set -x
    git submodule update --init
  )
fi
if [ ! -e examples/assemblyscript/node_modules ]; then
  ( set -x
    cd examples/assemblyscript
    npm install --no-save assemblyscript
  )
fi
( set -x
  cargo xtask build-applets
  cargo xtask --release build-applets
  cargo xtask build-runners
  cargo xtask --release build-runners
)
find . -mindepth 2 \
  -name test.sh -type f -perm /a+x \
  -print -not -execdir {} \; -quit
