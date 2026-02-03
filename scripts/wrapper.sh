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
. "$ROOT/scripts/system.sh"

CARGO_ROOT="$ROOT/.root"
export PATH="$CARGO_ROOT/bin:$PATH"

run() {
  [ "$WASEFIRE_WRAPPER_EXEC" = n ] && exit 0
  exec "$@"
}

ensure_cargo() {
  local flags="$1@$2"
  { cargo install --list --root="$CARGO_ROOT" | grep -q "^$1 v$2:\$"; } && return
  shift 2
  x cargo install --locked --root="$CARGO_ROOT" "$flags" "$@"
}

IS_CARGO=y
# This list is read and modified by scripts/upgrade.sh. It is also read by scripts/ci-cache.sh.
case "$1" in
  cargo)
    case "$2" in
      bloat) ensure_cargo cargo-bloat 0.12.1 ;;
      upgrade) ensure_cargo cargo-edit 0.13.8 ;;
      *) e "Wrapper does not support 'cargo $2'" ;;
    esac
    ;;
  defmt-print) ensure_cargo defmt-print 1.0.0 ;;
  mdbook) ensure_cargo mdbook 0.5.2 ;;
  nrfdfu) ensure_cargo nrfdfu 0.2.1 ;;
  probe-rs) ensure_cargo probe-rs-tools 0.31.0 ;;
  rust-objcopy|rust-size) ensure_cargo cargo-binutils 0.4.0 ;;
  taplo) ensure_cargo taplo-cli 0.10.0 ;;
  trunk) ensure_cargo trunk 0.21.14 ;;
  twiggy) ensure_cargo twiggy 0.8.0 ;;
  *) IS_CARGO=n ;;
esac
[ $IS_CARGO = y ] && run "$@"

has bin "$1" && run "$@"

# download <URL> [<chmod> [<name>]]
download() {
  local name="${3:-${1##*/}}"
  x curl -fLSso "$CARGO_ROOT/bin/$name" "$1"
  [ -z "$2" ] || x chmod "$2" "$CARGO_ROOT/bin/$name"
}

IS_LOCAL=y
case "$1" in
  bazel)
    URL=https://github.com/bazelbuild/bazelisk/releases/latest/download/bazelisk-linux-amd64
    download $URL +x bazel ;;
  uf2conv.py)
    URL=https://raw.githubusercontent.com/microsoft/uf2/refs/heads/master/utils
    download $URL/uf2conv.py +x
    download $URL/uf2families.json ;;
  *) IS_LOCAL=n ;;
esac
[ $IS_LOCAL = y ] && run "$@"

ensure bin "$1"
run "$@"
