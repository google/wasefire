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

EXTENSIONS='cff\|css\|git[a-z]*\|html\|json\|lock\|md\|pdf\|png\|svg\|toml'
EXTENSIONS="$EXTENSIONS"'\|wasm\|x\|yml'

for file in $(git ls-files \
  | grep -v -e '^third_party/' -e 'LICENSE$' -e '\.\('"$EXTENSIONS"'\)$'); do
  sed -n 'N;/Copyright/q;q1' $file || e "No copyright notice in $file"
done
