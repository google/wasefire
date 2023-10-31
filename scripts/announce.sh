#!/bin/bash
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
. scripts/package.sh

# This script aggregates the changelogs of a release commit.
#
# It must run on the release commit. The output is written as markdown into the
# docs/releases directory. An email should be sent to wasefire@googlegroups.com
# summarizing the change and linking to the markdown file.

DATE=$(git log -1 --pretty=%cs)
OUTPUT=docs/releases/$DATE.md
mkdir -p $(dirname $OUTPUT)

new() { sed -n 's/^## //p' CHANGELOG.md | sed -n 1p; }
dif() { sed -n '/^## /{:a;n;/^## /q;s/^#/##/;p;ba}' CHANGELOG.md; }
old() { sed -n 's/^## //p' CHANGELOG.md | sed -n 2p; }

CRATES=($(git show --oneline --stat -- '**/CHANGELOG.md' \
  | sed -n 's#^ \(.*\)/CHANGELOG.md.*#\1#p'))
rem() {
  local new=()
  local result=0
  for crate in ${CRATES[@]}; do
    if [ $1 = $crate ]; then
      result=1
    else
      new+=($crate)
    fi
  done
  CRATES=(${new[@]})
  return $result
}
ins() {
  local dir=crates/$1
  local has=n
  rem $dir || has=y
  ( cd $dir
    echo -n "### $(package_name) $(new) "
    case $has in
      n) echo "(no change)" ;;
      y)
        local old=$(old)
        if [ -n "$old" ]; then
          echo "(was $old)"
        else
          echo "(new)"
        fi
        dif
        ;;
    esac
  ) >> $OUTPUT
}
pri() {
  echo "$1" >> $OUTPUT
  echo >> $OUTPUT
}

rm -f $OUTPUT
pri "# Changes released on $DATE"
pri "## Applets"
ins prelude
ins api
pri "## Platforms"
ins board
ins scheduler
ins logger
pri "## Tools"
ins cli
pri "## Internal crates"
ins api-desc
ins api-macro
ins interpreter
ins store
ins stub
sed -i '$d' $OUTPUT

[ ${#CRATES[@]} -eq 0 ] || e "Did not process all crates: $CRATES"
