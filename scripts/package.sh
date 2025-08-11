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

package_name() { _package_string name; }
package_version() { _package_string version; }
package_edition() { _package_string edition; }
package_publish() { _package_raw publish; }
package_include() { _package_raw include; }
package_exclude() { _package_raw exclude; }
package_features() { sed -n '/^\[features]$/,/^$/{s/ = .*$//p}' Cargo.toml; }
package_doc_all_features() { _package_doc_raw all-features; }
package_doc_features() { _package_doc_raw features; }
package_doc_targets() { _package_doc_raw targets; }
package_doc_default_target() { _package_doc_raw default-target | tr -d '"'; }
package_bin_name() { _package_bin_string name; }
package_bin_path() { _package_bin_string path; }

cargo_info_version() { _cargo_info "$1" version; }

# Tested by ./scripts/publish.sh --dry-run
TOPOLOGICAL_ORDER='
one-of
logger
wire-derive
error
common
wire
wire/fuzz
sync
protocol
interpreter
store
store/fuzz
api-desc
api-desc/crates/update
api-macro
api
stub
prelude
board
scheduler
protocol-tokio
protocol-usb
cli-tools
cli
xtask
protocol/crates/schema
board/crates/syscall-test
runner-host/crates/web-common
runner-host/crates/web-client
runner-host/crates/web-server
runner-host
runner-nordic/crates/header
runner-nordic
runner-nordic/crates/bootloader
runner-opentitan/crates/register
runner-opentitan/crates/earlgrey
runner-opentitan
runner-opentitan/crates/earlgrey/crates/generate
wasm-bench
'

# Internal helpers
_package_raw() { sed -n '/^\[package]$/,/^$/{s/^'"$1"' = //p}' Cargo.toml; }
_package_string() { _package_raw "$1" | sed 's/^"\(.*\)"$/\1/'; }
_package_doc_raw() {
  sed -n '/^\[package\.metadata\.docs\.rs]$/,/^$/{s/^'"$1"' = //p}' Cargo.toml
}
_package_bin_string() { sed -n '/^\[\[bin\]\]$/,/^$/{s/^'"$1"' = "\(.*\)"$/\1/p}' Cargo.toml; }
_cargo_info() { cargo info --registry=crates-io -q "$1" | sed -n 's/^'"$2"': //p'; }
