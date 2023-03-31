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

if [ ! -e third_party/WebAssembly/spec/.git ]; then
  ( set -x
    git submodule update --init
  )
fi

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
ensure_bin() {
  [ $(has_bin $2) = y ] || add_missing $1
}
ensure_pkg() {
  pkg-config --exists $2 || add_missing $1
}

ensure_pkg libudev-dev libudev
ensure_pkg libusb-1.0-0-dev libusb-1.0
ensure_bin npm npm
ensure_bin wabt wasm-strip
ensure_bin binaryen wasm-opt

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

MISSING=
ensure_bin cargo-binutils rust-size
ensure_bin cargo-bloat cargo-bloat
ensure_bin probe-run probe-run
ensure_bin taplo-cli taplo
ensure_bin mdbook mdbook

if [ -n "$MISSING" ]; then
  ( set -x
    cargo install$MISSING
  )
fi

if [ ! -e examples/assemblyscript/node_modules ]; then
  ( set -x
    cd examples/assemblyscript
    npm install --no-save assemblyscript
  )
fi
