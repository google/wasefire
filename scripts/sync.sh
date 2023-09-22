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

cargo xtask update-apis

book_example() {
  sed '/ANCHOR/d' book/src/applet/prelude/$1.rs > examples/rust/$2/src/lib.rs
}

book_example led blink
book_example button1 button
book_example button2 led
book_example timer button_abort
book_example usb memory_game
book_example store store

# This is done here instead of upgrade.sh for 2 reasons:
# 1. This runs more often so users would install with the latest script.
# 2. The upgrade.sh would need a way to know the latest version.
RUSTUP_CURL="curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs"
RUSTUP_SCRIPT="$(curl -s https://rustup.rs \
  | sed -n '/^<div id="platform-instructions-unix"/,/^<\/div>$/'\
'{s#^ *<pre class="rustup-command">\(.*\)</pre>$#\1#p;T;q}'
)"
[ "$RUSTUP_SCRIPT" = "$RUSTUP_CURL | sh" ] || e "RUSTUP_CURL is out of sync"
eval "$RUSTUP_CURL" \
  | diff - third_party/rust-lang/rustup/rustup-init.sh >/dev/null \
  || e 'rustup submodule is out of sync'
