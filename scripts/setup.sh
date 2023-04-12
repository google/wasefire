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

# This script installs any missing dependency on a best effort basis. It is
# idempotent and may be run to check whether everything is set up.

has_bin() { which $1 >/dev/null 2>&1; }

has_bin rustup || e "Missing rustup. Install from https://rustup.rs"

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
  has_bin apt-get || e "Unsupported system. Install:$MISSING"
  x sudo apt-get install$MISSING
fi
