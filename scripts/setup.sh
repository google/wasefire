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
  if [ -n "$WASEFIRE_YES" ]; then
    RUSTUP_ARGS='-s -- -y --default-toolchain=none --profile=minimal --no-modify-path'
  fi
  x curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | x sh $RUSTUP_ARGS
fi

# Transitive dependencies of xtask.
ensure bin cc
ensure lib libudev
ensure lib openssl

# Transitive dependencies of runner-host.
ensure bin usbip
ensure apt gcc-multilib
ensure apt libssl-dev:i386
