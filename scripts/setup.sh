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

# This script installs any missing dependency on a best effort basis. It is
# idempotent and may be run to check whether everything is set up.

has_bin() {
  if which $1 >/dev/null 2>&1; then
    echo y
  else
    echo n
  fi
}

if [ $(has_bin cargo) = n ]; then
  echo "Install rustup from https://rustup.rs"
  exit 1
fi

MISSING=
add_missing() {
  MISSING="$MISSING $1"
}
ensure_pkg() {
  pkg-config --exists $2 || add_missing $1
}

ensure_pkg libudev-dev libudev
ensure_pkg libusb-1.0-0-dev libusb-1.0

if [ -n "$MISSING" ]; then
  if [ $(has_bin apt-get) = y ]; then
    ( set -x
      sudo apt-get update
      sudo apt-get install$MISSING
    )
  else
    echo "Install the following packages:$MISSING"
    exit 1
  fi
fi

if [ ! -e examples/assemblyscript/node_modules ]; then
  ( set -x
    cd examples/assemblyscript
    ../../scripts/wrapper.sh npm install --no-save assemblyscript
  )
fi
