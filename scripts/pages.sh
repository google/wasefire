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

# This script synchronizes the gh-pages branch from a clean main.

[ -z "$(git status -s)" ] || e 'not clean'
[ -n "$CI" -o "$(git symbolic-ref -q HEAD)" = refs/heads/main ] || e 'not main'
COMMIT="$(git rev-parse -q --verify HEAD)"
[ -n "$COMMIT" ] || e 'failed to get commit hash'

git diff --quiet "$(git log --pretty=format:%f origin/gh-pages)".. -- book \
  && d "origin/gh-pages is already up-to-date"

WASEFIRE_WRAPPER_EXEC=n ./scripts/wrapper.sh mdbook
( cd book
  ../scripts/wrapper.sh mdbook build 2>/dev/null )
mv book/book html

git show-ref -q --verify refs/heads/gh-pages && git branch -qD gh-pages
git checkout -q --orphan gh-pages
git rm -qrf .
git clean -qfxde/html
find html -mindepth 1 -maxdepth 1 -exec mv {} . \;
rmdir html
git add .
git commit -qm"$COMMIT"
git checkout -q main
d "gh-pages has been updated"
