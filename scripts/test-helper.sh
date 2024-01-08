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

GIT_ROOT="$(git rev-parse --show-toplevel)"
SELF="$0"

. "$GIT_ROOT"/scripts/log.sh
. "$GIT_ROOT"/scripts/package.sh

ensure_applet() {
  ( cd "$GIT_ROOT"
    if [ ! -e target/wasefire/applet.wasm ]; then
      mkdir -p target/wasefire
      x touch target/wasefire/applet.wasm
    fi
    if [ ! -e target/wasefire/libapplet.a ]; then
      x cargo xtask --native-target=thumbv7em-none-eabi applet rust hello
    fi
  )
}

ensure_submodule() {
  ( cd "$GIT_ROOT"
    [ -e "$1/.git" ] || x git submodule update --init "$1"
  )
}

# check_*_api <prefix> <features> <clippy-args>..
# <prefix> = "api-", "applet-api-", or "board-api-"
# <features> = "--features=" or "--features=wasm,std,"
# <clippy-args> = "--all-targets" or "--target=wasm32-unknown-unknown"
check_applet_api() { _test_check_api "$_TEST_APPLET_API" "$@"; }
check_board_api() { _test_check_api "$_TEST_BOARD_API" "$@"; }

test_helper() {
  _test_desc | grep -v -e '^$' -e '^#' -e 'cargo \(check\|test\)' \
    && e 'Invalid description (invalid commands are listed above).'
  _test_desc | _test_check | grep 'cargo check' | sh -ex
  _test_desc | grep 'cargo test' | sh -ex
  x cargo fmt -- --check
  _test_desc | _test_check | _test_clippy | grep 'cargo clippy' | sh -ex
  if [ -e src/lib.rs -a "$(package_publish)" = true ]; then
    features=$(package_doc_features | tr -d '[]" ')
    [ -n "$features" ] && features="--features=$features"
    target="$(package_doc_default_target)"
    [ -z "$(package_doc_targets)" ] || e 'docs.rs targets unsupported'
    [ -n "$target" ] && target="--target=$target"
    x env RUSTDOCFLAGS=--deny=warnings cargo doc --no-deps $target $features
  fi
  exit
}

_test_desc() {
  sed '0,/^test_helper$/d;:a;/\\$/{N;s/\\\n//;ta};s/ \+/ /g' "$SELF"
}

_test_check() { sed 's/cargo test/cargo check --all-targets/'; }
_test_clippy() { sed 's/cargo check/cargo clippy/;s/$/ -- --deny=warnings/'; }

_test_check_api() {
  local api="$1"
  local prefix="$2"
  local features="$3"
  shift 3
  local full="full-${prefix%-}"
  _test_diff 'the API features' "$api" \
    $(package_features | sed -n "s/^$prefix//p")
  if package_features | grep -q "^$full\$"; then
    _test_diff "the $full feature dependencies" "$api" \
      $(_test_full_deps $full $prefix)
  fi
  for api in $api; do
    x cargo clippy "$@" "$features$prefix$api" -- --deny=warnings
  done
}

_test_full_deps() {
  local full=$1
  local prefix=$2
  sed -n '/^'$full' = \[$/,/^]$/{s/^  "'$prefix'\(.*\)",/\1/p}' Cargo.toml
}

_test_diff() {
  local where="$1"
  shift
  local x
  for x in $1; do
    shift
    [ $# -eq 0 ] && e "$x is missing from $where"
    [ $x = $1 ] && continue
    if [ $x = "$(_test_min $x $1)" ]; then
      e "$x is missing from $where"
    else
      e "$1 is present in $where but is unexpected"
    fi
  done
  shift
  [ $# -eq 0 ] || e "$1 is present in $where but is unexpected"
}

_test_min() { { echo "$1"; echo "$2"; } | sort | head -n1; }

_TEST_APPLET_API='
button
crypto-ccm
crypto-ec
crypto-gcm
crypto-hash
crypto-hkdf
crypto-hmac
gpio
led
platform
platform-update
radio-ble
rng
store
store-fragment
timer
uart
usb-serial
'

_TEST_BOARD_API='
button
crypto-aes128-ccm
crypto-aes256-gcm
crypto-hmac-sha256
crypto-hmac-sha384
crypto-p256
crypto-p384
crypto-sha256
crypto-sha384
gpio
led
platform
platform-update
radio-ble
rng
storage
timer
uart
usb-serial
'
