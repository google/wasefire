#!/bin/sh
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

set -e
. scripts/log.sh

# This script checks that all source files have a copyright notice in the first
# 2 lines.

for file in $(git ls-files ':(attr:textreview)'); do
  case "$file" in
    *.gitignore|.git*|LICENSE|*/LICENSE) continue ;;
    *.cff|*.css|*.html|*.json|*.lock|*.md|*.svg|*.toml|*.txt|*.x|*.yml) continue ;;
    crates/cli-tools/src/data/lib.rs) continue ;;
  esac
  sed -n 'N;/Copyright/q;q1' "$file" || e "No copyright notice in $file"
done
