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
      bin)
        case "$2" in
          cc) set build-essential ;;
          curl) set curl ;;
          npm)
            # AssemblyScript needs Node 16 or later.
            curl -fsSL https://deb.nodesource.com/setup_16.x | sudo -E bash -
            set nodejs
            ;;
          pkg-config) set pkgconf ;;
          usbip) set usbip ;;
          wasm-opt) set binaryen ;;
          wasm-strip) set wabt ;;
          *) e "Internal error: _install_apt unimplemented for $*" ;;
        esac ;;
      lib)
        case "$2" in
          libudev) set libudev-dev ;;
          libusb-1.0) set libusb-1.0-0-dev ;;
          *) e "Internal error: _install_apt unimplemented for $*" ;;
        esac ;;
    esac
    x sudo apt-get install "$1"
  else
    e "Unsupported system. Install $1 '$2' and rerun the command."
  fi
}
