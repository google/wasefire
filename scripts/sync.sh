#!/bin/sh
# Copyright 2022 Google LLC
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

# This script synchronizes generated content.

[ "$1" = --force ] && FORCE=y

update_api() {
  cargo update-api --features=wasefire-applet-api-desc/full-api -- \
    --lang=$1 --output=examples/$1/api.$2
}
update_api assemblyscript ts

add_lint() { echo "$3 = \"$2\"" >> $1; }
for dir in $(find crates -name Cargo.toml -printf '%h\n' | sort); do
  file=$dir/Cargo.toml
  crate=${dir#crates/}
  grep -q '^\[lints\.' $file && e "unexpected [lints.*] section in $file"
  sed -i '/^\[lints\]$/q' $file
  [ "$(tail -n1 $file)" = '[lints]' ] || printf '\n[lints]\n' >> $file
  add_lint $file allow clippy.unit-arg
  # add_lint $file warn rust.elided-lifetimes-in-paths
  # add_lint $file warn rust.missing-debug-implementations
  # TODO: Use the same [ -e src/lib.rs -a "$(package_publish)" = true ] test as in test-helper.
  case $crate in
    board|prelude) add_lint $file warn rust.missing-docs ;;
  esac
  # TODO: Enable for all crates.
  case $crate in
    interpreter|runner-*|scheduler|xtask|*/fuzz) ;;
    *) add_lint $file warn rust.unreachable-pub ;;
  esac
  add_lint $file warn rust.unsafe-op-in-unsafe-fn
  case $crate in
    */fuzz) ;;
    *) add_lint $file warn rust.unused-crate-dependencies ;;
  esac
  # add_lint $file warn rust.unused-results
done

( cd crates/protocol/crates/schema
  cargo run --features=host
  cargo run --features=device
)

book_example() {
  local src=book/src/applet/prelude/$1.rs
  local dst=examples/rust/$2/src/lib.rs
  # We only check that the destination is newer by more than one second, because when cloning the
  # repository or switching branches, it may happen that the destination is slightly newer.
  if [ -z "$FORCE" -a $(stat -c%Y $dst) -gt $(($(stat -c%Y $src) + 1)) ]; then
    t "Update $src instead of $dst"
    i "If you switched branch and did not modify those files, rerun with --force"
    e "$dst seems to have been manually modified"
  fi
  # Besides removing all anchors, we insert a warning before the #![no_std] line, which all examples
  # should have near the beginning of the file.
  sed '/ANCHOR/d;/^#!\[no_std\]$/{i \'"
// DO NOT EDIT MANUALLY:\\
// - Edit $src instead.\\
// - Then use ./scripts/sync.sh to generate this file.\\

}" $src > $dst
  # Now that the destination has been updated, it is newer than the source. So we touch the source
  # to preserve the invariant that the destination is never newer than the source.
  touch $src
}

book_example led blink
book_example button1 button
book_example button2 led
book_example timer button_abort
book_example usb memory_game
book_example store store

GIT_MODULES='
SchemaStore/schemastore
WebAssembly/spec
rust-lang/rustup
wasm3/wasm-coremark
'
[ "$(echo "$GIT_MODULES" | sort | tail -n+2)" = "$(echo "$GIT_MODULES")" ] \
  || e "GIT_MODULES is not sorted"
for m in $GIT_MODULES; do
  echo "[submodule \"third_party/$m\"]"
  printf "\tpath = third_party/$m\n"
  printf "\turl = https://github.com/$m.git\n"
done > .gitmodules

# This is done here instead of upgrade.sh for 2 reasons:
# 1. This runs more often so users would install with the latest script.
# 2. The upgrade.sh would need a way to know the latest version.
RUSTUP_CURL="curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs"
RUSTUP_SCRIPT="$(curl -s https://rustup.rs \
  | sed -n '/^<div id="platform-instructions-unix"/,/^<\/div>$/'\
'{s#^ *<pre class="rustup-command">\(.*\)</pre>$#\1#p;T;q}'
)"
[ "$RUSTUP_SCRIPT" = "$RUSTUP_CURL | sh" ] || e "RUSTUP_CURL is out of sync"
git submodule update --init third_party/rust-lang/rustup
eval "$RUSTUP_CURL" \
  | diff - third_party/rust-lang/rustup/rustup-init.sh >/dev/null \
  || e 'rustup submodule is out of sync'
