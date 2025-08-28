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

has() {
  case "$1" in
    apt) dpkg -l "$2" 2>/dev/null | grep "^ii  $2" >/dev/null ;;
    bin) which "$2" >/dev/null 2>&1 ;;
    lib) pkg-config --exists "$2" ;;
    *) e "Internal error: expected bin or lib, got $1" ;;
  esac
}

ensure() {
  has "$1" "$2" || install "$1" "$2"
}

install() {
  if has bin apt-get; then
    case "$1" in
      apt) set "$2" ;;
      bin)
        case "$2" in
          cc) set build-essential ;;
          curl) set curl ;;
          mdl) set markdownlint ;;
          npm) set npm ;;
          pkg-config) set pkgconf ;;
          socat) set socat ;;
          usbip)
            case "$(_system_dist_id)" in
              Ubuntu) set linux-tools-common ;;
              *) set usbip ;;
            esac ;;
          wasm-opt) set binaryen ;;
          wasm-strip) set wabt ;;
          *) e "Internal error: apt-get install unimplemented for $*" ;;
        esac ;;
      lib)
        case "$2" in
          libudev) set libudev-dev ;;
          openssl) set libssl-dev ;;
          *) e "Internal error: apt-get install unimplemented for $*" ;;
        esac ;;
    esac
    if ! dpkg --print-foreign-architectures | grep -q i386; then
      x sudo dpkg --add-architecture i386
      x sudo apt-get update
    fi
    # Make sure apt-get update has run at least once (useful for fresh VMs).
    [ -e /var/lib/apt/lists/lock ] || x sudo apt-get update
    x sudo apt-get install $WASEFIRE_YES "$1"
  else
    e "Unsupported system. Install $1 '$2' and rerun the command."
  fi
}

_system_dist_id() { lsb_release -is; }
