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

RUSTUP_CMD="$(curl -s https://rustup.rs \
  | sed -n '/^<div id="platform-instructions-unix"/,/^<\/div>$/'\
'{s#^ *<pre class="rustup-command">\(.*\)</pre>$#\1#p;T;q}'
)"
sed -i '\#^  i ".*https://rustup.rs"$#{n;i\
'"  $RUSTUP_CMD"'
;d}' scripts/setup.sh
