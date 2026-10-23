#!/bin/sh
# Copyright 2026 Google LLC
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
. scripts/shard.sh

# This script runs cargo audit on all crates.

[ "$1" = --fix ] && { shift; fix=y; }

shard_init "$@"
for lock in $(git ls-files '*/Cargo.lock'); do
  shard_next || continue
  code=0
  x ./scripts/wrapper.sh cargo audit $no_fetch --file="$lock" || code=$?
  if [ $code -ne 0 ]; then
    [ "$fix" = y ] || exit $code
    x ./scripts/wrapper.sh cargo audit fix --file="$lock"
  fi
  no_fetch=--no-fetch
done
shard_done
