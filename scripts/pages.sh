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
[ "$(git symbolic-ref -q HEAD)" = refs/heads/main ] || e 'not main'

( cd book
  mdbook build 2>/dev/null )
mv book/book html

git show-ref -q --verify refs/heads/gh-pages && git branch -qD gh-pages
git checkout -q --orphan gh-pages
git rm -qrf .
git clean -qfxde/html
mv html/* html/.* .
rmdir html
git add .
git commit -qm"$(git rev-parse -q --verify main)"
git checkout -q main
