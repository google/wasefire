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

# This script checks that all source files have a correct copyright notice in the first 2 lines.

# Checks and prints the copyright notice of a file (normalizing the comment prefix).
notice() {
  local file="$1"
  local start prefix
  start=$(sed -n '/ Copyright/{=;q}' "$file")
  [ "${start:-3}" -lt 3 ] || return
  prefix="$(sed -n "$start"'s/ Copyright.*//p' "$file")"
  case "$prefix" in
    '//') prefix='\/\/' ;;
  esac
  sed -n "$start"'s/\(Copyright\) ..../\1/;'\
"$start"',+13{/^\('"$prefix"'\|$\)/!d;s/^'"$prefix"'/#/;p}' "$file"
}

REFERENCE_NOTICE="$(notice $0)"

i "Checking copyright notices (may take some time)"
for file in $(git ls-files ':(attr:textreview)'); do
  case "$file" in
    *.gitignore|.git*|LICENSE|*/LICENSE) continue ;;
    *.cff|*.css|*.html|*.json|*.lock|*.md|*.scss|*.svg|*.toml|*.txt|*.x|*.yaml|*.yml) continue ;;
    crates/cli-tools/src/data/lib.rs) continue ;;
  esac
  echo -n .
  [ "$(notice $file)" = "$REFERENCE_NOTICE" ] || e "Wrong copyright notice in $file"
done
echo done

[ -z "$GITHUB_BASE_REF" ] && exit
expected=$(date +%Y)
for file in $(git diff "origin/$GITHUB_BASE_REF" --summary | grep '^ create ' | cut -f5 -d' '); do
  [ -d $file ] && continue
  actual=$(sed -n 'N;s/^.*Copyright \(....\).*$/\1/p;q' "$file")
  [ -n "$actual" ] || continue
  line=$(grep -n Copyright $file | cut -f1 -d:)
  [ $actual = $expected ] && continue
  message="Expected Copyright $expected (ignore if file was moved or copied)."
  echo "::warning file=$file,line=$line::$message"
done
