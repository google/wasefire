#!/bin/sh
# Copyright 2022 Google LLC
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

# This script runs the continuous integration tests.

K=${1:-0}
M=${2:-1}

i=$(( M - 1 ))
for dir in $(git ls-files '*/test.sh'); do
  i=$(( (i + 1) % M ))
  [ $i -eq $K ] || continue
  grep -q test-helper $dir || e "$dir doesn't use test-helper.sh"
  dir=$(dirname $dir)
  i "Run tests in $dir"
  ( cd $dir && ./test.sh )
done
