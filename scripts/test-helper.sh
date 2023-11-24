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

test_helper() {
  _test_desc_raw | grep -v -e '^$' -e '^#' -e 'cargo \(check\|test\)' \
    && e 'Invalid description (invalid commands are listed above).'
  _test_desc '\(check\|test\)' check
  _test_desc test test
  x cargo fmt -- --check
  _test_desc '\(check\|test\)' clippy ' -- --deny=warnings'
  if [ -e src/lib.rs -a "$(package_publish)" = true ]; then
    features=$(package_doc_features | tr -d '[]" ')
    [ -n "$features" ] && features="--features=$features"
    target="$(package_doc_default_target)"
    [ -z "$(package_doc_targets)" ] || e 'docs.rs targets unsupported'
    [ -n "$target" ] && target="--target=$target"
    x env RUSTDOCFLAGS=--deny=warnings cargo doc $target $features
  fi
  exit
}

_test_desc_raw() {
  sed '0,/^test_helper$/d;:a;/\\$/{N;s/\\\n//;ta};s/ \+/ /g' "$SELF"
}

_test_desc() {
  _test_desc_raw | sed -n "s/cargo $1/cargo $2/p" | sed "s/\$/$3/" | sh -ex
}
