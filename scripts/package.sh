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

package_name() { package__string name; }
package_version() { package__string version; }
package_publish() { package__raw publish; }
package_include() { package__raw include; }
package_exclude() { package__raw exclude; }
package_features() { sed -n '/^\[features]$/,/^$/{s/ = .*$//p}' Cargo.toml; }

# Internal helpers
package__raw() { sed -n '/^\[package]$/,/^$/{s/^'"$1"' = //p}' Cargo.toml; }
package__string() { package__raw "$1" | sed 's/^"\(.*\)"$/\1/'; }
