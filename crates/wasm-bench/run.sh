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

. "$(git rev-parse --show-toplevel)"/scripts/test-helper.sh

ensure_submodule third_party/wasm3/wasm-coremark

BUILD=n
DEBUG=n
while [ "${1#--}" != "$1" ]; do
  case "${1#--}" in
    build) BUILD=y ;;
    debug) DEBUG=y ;;
    *) e "Unsupported flag --$1" ;;
  esac
  shift
done

case "$1" in
  linux)
    TARGET="$(rustc -vV | sed -n 's/^host: //p')"
    RUN=
    ;;
  nordic)
    TARGET=thumbv7em-none-eabi
    RUSTFLAGS=-Clink-arg=-Tlink.x
    RUN='../../scripts/wrapper.sh probe-rs run --chip=nRF52840_xxAA'
    ;;
  *) e "Unsupported target: $1" ;;
esac

case "$2" in
  base) ;;
  wasmtime) ;;
  *) e "Unsupported runtime: $2" ;;
esac

case "$3" in
  perf) PROFILE=release ;;
  size) PROFILE=release-size ;;
  *) e "Unsupported profile: $3" ;;
esac

if [ $1 != linux ]; then
  if [ $2 = wasmtime ]; then
    RUSTFLAGS="$RUSTFLAGS --cfg=pulley_disable_interp_simd"
  fi
  BUILD_STD='-Zbuild-std=core,alloc -Zbuild-std-features=panic_immediate_abort'
  if [ $3 = size ]; then
    BUILD_STD="$BUILD_STD,optimize_for_size"
  fi
  RUSTFLAGS="$RUSTFLAGS --allow=unused-crate-dependencies"
fi

if [ $DEBUG = y ]; then
  BUILD_STD=
  DEBUG_CONFIG=--config=profile.release.debug=true
fi

x env RUSTFLAGS="$RUSTFLAGS" cargo build $DEBUG_CONFIG \
  --target=$TARGET --profile=$PROFILE $BUILD_STD --features=target-$1,runtime-$2

ELF=../../target/$TARGET/$PROFILE/wasm-bench
x ../../scripts/wrapper.sh rust-size $ELF
[ $BUILD = y ] || x $RUN $ELF
