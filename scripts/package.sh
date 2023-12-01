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
package_publish() { _package_raw publish; }
package_include() { _package_raw include; }
package_exclude() { _package_raw exclude; }
package_features() { sed -n '/^\[features]$/,/^$/{s/ = .*$//p}' Cargo.toml; }
package_doc_features() { _package_doc_raw features; }
package_doc_targets() { _package_doc_raw targets; }
package_doc_default_target() { _package_doc_raw default-target | tr -d '"'; }

# Internal helpers
_package_raw() { sed -n '/^\[package]$/,/^$/{s/^'"$1"' = //p}' Cargo.toml; }
_package_string() { _package_raw "$1" | sed 's/^"\(.*\)"$/\1/'; }
_package_doc_raw() {
  sed -n '/^\[package\.metadata\.docs\.rs]$/,/^$/{s/^'"$1"' = //p}' Cargo.toml
}
