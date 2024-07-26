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
# summarizing the changes with links to the release file, relevant PRs, and
# relevant documentation on docs.rs.

DATE=$(git log -1 --pretty=%cs)
OUTPUT=docs/releases/$DATE.md
mkdir -p $(dirname $OUTPUT)

new() { sed -n 's/^## //p' CHANGELOG.md | sed -n 1p; }
dif() { sed -n '/^## /{:a;n;/^## /q;s/^#/##/;p;ba}' CHANGELOG.md; }
old() { sed -n 's/^## //p' CHANGELOG.md | sed -n 2p; }
get-pr() { git log -1 $1 --pretty=%s | sed 's/^.*(#\([0-9]*\))$/\1/'; }
format-pr() { echo "[#$1](https://github.com/google/wasefire/pull/$1)"; }
format-dif() {
  i=$(grep -n '^## ' CHANGELOG.md | sed 's/:.*$//;q')
  dif | while IFS= read -r line; do
    (( i += 1 ))
    case "$line" in
      '- '*)
        commit=$(git blame -ls -L$i,+1 CHANGELOG.md | cut -f1 -d' ')
        echo "- $(format-pr $(get-pr $commit))"
        line=" ${line#-}"
        ;;
    esac
    printf '%s\n' "$line"
  done
}

CRATES=($(git show --oneline --stat -- '*/CHANGELOG.md' \
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
  [ -d $dir ] || return 0
  ( cd $dir
    echo -n "### $(package_name) $(new) "
    case $has in
      n) echo "(no change)"; echo ;;
      y)
        local old=$(old)
        if [ -n "$old" ]; then
          echo "(was $old)"; format-dif
        else
          echo "(new)"; echo
        fi
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
pri "This release was cut by $(format-pr $(get-pr))."
pri "## Applets"
ins prelude
ins api
pri "## Platforms"
ins board
ins scheduler
ins logger
pri "## Common crates"
ins error
ins sync
pri "## Tools"
ins cli
pri "## Internal crates"
ins api-desc
ins api-macro
ins cli-tools
ins interpreter
ins protocol
ins protocol-usb
ins store
ins stub
ins wire
ins wire-derive
sed -i '$d' $OUTPUT

[ ${#CRATES[@]} -eq 0 ] || e "Did not process all crates: $CRATES"
