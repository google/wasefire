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

# This script runs the provided command possibly installing it if needed.

ROOT="$(dirname "$0")"
# We don't support running from the scripts directory itself.
[ "${ROOT%scripts}" != "$ROOT" ]
ROOT="${ROOT%/scripts}"

. "$ROOT/scripts/log.sh"

CARGO_ROOT="$ROOT/.root"

run() {
  [ "$WASEFIRE_WRAPPER_EXEC" = n ] && exit 0
  exec "$@"
}

ensure_cargo() {
  if ! cargo install --list --root="$CARGO_ROOT" | grep -q "^$1 v$2:\$"; then
    PATH="$CARGO_ROOT/bin:$PATH" x cargo install --root="$CARGO_ROOT" "$1@$2"
  fi
}

IS_CARGO=y
# This list is read and modified by scripts/upgrade.sh.
case "$1" in
  cargo)
    case "$2" in
      bloat) ensure_cargo cargo-bloat 0.11.1 ;;
      *) e "Wrapper does not support 'cargo $2'" ;;
    esac
    ;;
  mdbook) ensure_cargo mdbook 0.4.28 ;;
  probe-run) ensure_cargo probe-run 0.3.8 ;;
  rust-size) ensure_cargo cargo-binutils 0.3.6 ;;
  taplo) ensure_cargo taplo-cli 0.8.0 ;;
  *) IS_CARGO=n ;;
esac
[ $IS_CARGO = y ] && PATH="$CARGO_ROOT/bin:$PATH" run "$@"

has_bin() {
  which "$1" >/dev/null 2>&1
}

has_bin "$1" && run "$@"

has_bin apt-get || e 'Wrapper only supports apt-get.'

ensure_bin() {
  x sudo apt-get install "$1"
}

case "$1" in
  npm) ensure_bin npm ;;
  wasm-opt) ensure_bin binaryen ;;
  wasm-strip) ensure_bin wabt ;;
  *) e "Wrapper does not support '$1'" ;;
esac
run "$@"
