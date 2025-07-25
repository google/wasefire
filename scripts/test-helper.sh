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
    for file in applet.wasm libapplet.a; do
      if [ ! -e target/wasefire/$file ]; then
        mkdir -p target/wasefire
        x touch target/wasefire/$file
      fi
    done
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
# <clippy-args> = "--target=wasm32-unknown-unknown"
check_applet_api() { _test_check_api "$_TEST_APPLET_API" "$@"; }
check_board_api() { _test_check_api "$_TEST_BOARD_API" "$@"; }
# check_software_crypto <features> <clippy-args>..
# <features> = "--features=" or "--features=wasm,std,"
# <clippy-args> = "--target=wasm32-unknown-unknown"
check_software_crypto() {
  local features="$1"
  shift 1
  _test_diff 'the software crypto features' "$_TEST_SOFTWARE_CRYPTO" \
    $(package_features | sed -n "s/^software-crypto-//p")
  if package_features | grep -q '^internal-software-crypto$'; then
    _test_diff "the internal-software-crypto feature dependencies" \
      "$_TEST_SOFTWARE_CRYPTO" \
      $(_test_full_deps internal-software-crypto software-crypto-)
  fi
  for feat in $_TEST_SOFTWARE_CRYPTO; do
    x cargo clippy --lib "$@" "${features}software-crypto-$feat" -- --deny=warnings
  done
}

# diff_sorted <name> <left> <right>..
# <name> is the name of the <right> list
# <left> is a sorted list after word expansion
# <right>.. is a sorted list
diff_sorted() {
  _test_diff "$@"
}

test_helper() {
  _test_desc | grep -Ev 'cargo (check|(miri )?test|run) --(lib|(bin|test|example)=[^ ]*)( |$)' \
    && e 'Invalid description (invalid commands are listed above).'
  _test_ensure_lib
  _test_ensure_bins
  _test_ensure_dir tests test test
  _test_ensure_dir examples check example
  _test_desc | _test_check | grep 'cargo check' | sh -ex
  _test_desc | grep 'cargo test' | sh -ex
  _test_desc | grep 'cargo miri test' | sh -ex
  _test_desc | grep 'cargo run' | sh -ex
  x cargo fmt -- --check
  _test_desc | _test_check | _test_clippy | grep 'cargo clippy' | sh -ex
  if [ -e src/lib.rs -a "$(package_publish)" = true ]; then
    features=$(package_doc_features | tr -d '[]" ')
    [ -n "$features" ] && features="--features=$features"
    [ "$(package_doc_all_features)" = true ] && features=--all-features
    target="$(package_doc_default_target)"
    [ -z "$(package_doc_targets)" ] || e 'docs.rs targets unsupported'
    [ -n "$target" ] && target="--target=$target"
    [ -n "$target" ] || x cargo test --doc $features
    x env RUSTDOCFLAGS=--deny=warnings cargo doc --no-deps $target $features
  fi
  exit
}

_test_desc() {
  sed '0,/^test_helper$/d;:a;/\\$/{N;s/\\\n//;ta};s/ \+/ /g' "$SELF" | grep -Ev '^($|#)'
}

_test_check() { sed 's/cargo test/cargo check --profile=test/;s/cargo run/cargo check/'; }
_test_clippy() { sed 's/cargo check/cargo clippy/;s/$/ -- --deny=warnings/'; }

_test_ensure_lib() {
  if [ -e src/lib.rs ]; then
    if git grep -q '#\[test\]' src
    then _test_ensure_desc 'cargo test --lib'
    else _test_ensure_desc 'cargo (check|test) --lib'
    fi
  fi
}
_test_ensure_bins() {
  local i
  for i in $(package_bin_name); do
    _test_ensure_desc "cargo (check|test|run) --bin=$i"
  done
  if [ -e src/main.rs ] && ! package_bin_path | grep -q src/main.rs; then
    _test_ensure_desc "cargo (check|test|run) --bin=$(package_name)"
  fi
}
_test_ensure_dir() {
  local i
  [ -d $1 ] || return 0
  for i in $(find $1 -name '*.rs' -printf '%P\n'); do
    _test_ensure_desc "cargo $2 --$3=${i%.rs}"
  done
}
_test_ensure_desc() { _test_desc | grep -Eq "$1" || e "No \`$1\` found."; }

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
    x cargo clippy --lib "$@" "$features$prefix$api" -- --deny=warnings
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
clock
crypto-cbc
crypto-ccm
crypto-ec
crypto-ecdh
crypto-ecdsa
crypto-ed25519
crypto-gcm
crypto-hash
crypto-hkdf
crypto-hmac
gpio
led
platform
platform-protocol
platform-update
radio-ble
rng
store
store-fragment
timer
uart
usb-ctap
usb-serial
'

_TEST_BOARD_API='
button
clock
crypto-aes128-ccm
crypto-aes256-cbc
crypto-aes256-gcm
crypto-ed25519
crypto-hmac-sha256
crypto-hmac-sha384
crypto-p256
crypto-p256-ecdh
crypto-p256-ecdsa
crypto-p384
crypto-p384-ecdh
crypto-p384-ecdsa
crypto-sha256
crypto-sha384
gpio
led
radio-ble
rng
storage
timer
uart
usb-ctap
usb-serial
'

_TEST_SOFTWARE_CRYPTO='
aes128-ccm
aes256-cbc
aes256-gcm
ed25519
hmac-sha256
hmac-sha384
p256
p256-ecdh
p256-ecdsa
p384
p384-ecdh
p384-ecdsa
sha256
sha384
'
