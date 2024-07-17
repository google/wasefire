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
echo "heloo####################"
printenv

set -e

# This script installs the dependencies to run xtask. Any other dependencies
# should be installed on first usage only.
#
# This script only supports apt-get systems at the moment. For other systems,
# the script may fail with an error indicating the missing binary or library.
# The user needs to manually install it and rerun the script again. This may
# need to be repeated until the script exits successfully.
#
# This script is idempotent and may be cheaply run to check whether everything
# is set up. It won't modify anything if that's the case.

. scripts/log.sh
. scripts/system.sh

# Parse flags
while [ $# -gt 0 ]; do
  case $1 in
    -h) echo "Usage: $0 [-y]"; exit ;;
    -y) shift; export WASEFIRE_YES=-y ;;
    -*) e "Unknown flag '$1'" ;;
    *) e "Unexpected argument '$1'" ;;
  esac
done

# Basic binaries used for all Unix systems.
ensure bin curl
ensure bin pkg-config

if ! has bin rustup; then
  x git submodule update --init third_party/rust-lang/rustup
  if [ -n "$WASEFIRE_YES" ]; then
    x ./third_party/rust-lang/rustup/rustup-init.sh -y \
      --default-toolchain=none --profile=minimal --no-modify-path
  else
    x ./third_party/rust-lang/rustup/rustup-init.sh
  fi
fi

# Transitive dependencies of xtask.
ensure bin cc
ensure lib libudev

# Transitive dependencies of runner-host. This should ideally be installed on
# demand by xtask.
ensure bin usbip
