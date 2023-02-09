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

sync() {
  sed '/ANCHOR/d' "src/applet/prelude/$1.rs" > "../examples/rust/$2/src/lib.rs"
}

sync led blink
sync button1 button
sync button2 led
sync timer button_abort
sync usb memory_game
sync store store
