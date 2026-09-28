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

# Known informational advisories in transitive dependencies that require
# upstream migrations rather than lockfile updates. Keep these explicit so
# --deny warnings still catches any new advisory.
IGNORED_ADVISORIES="
RUSTSEC-2023-0089
RUSTSEC-2024-0370
RUSTSEC-2024-0436
RUSTSEC-2025-0141
RUSTSEC-2026-0110
"

ignore_args=
for advisory in $IGNORED_ADVISORIES; do
  ignore_args="$ignore_args --ignore $advisory"
done

# Audit every committed Rust lockfile. The first invocation refreshes the
# advisory database; subsequent checks reuse it to avoid dozens of fetches.
first=true
for lock in $(git ls-files | grep 'Cargo.lock$' | sort); do
  no_fetch=--no-fetch
  if [ "$first" = "true" ]; then
    no_fetch=
    first=false
  fi
  # shellcheck disable=SC2086  # ignore_args intentionally expands into arguments.
  x ./scripts/wrapper.sh cargo audit --deny warnings $ignore_args $no_fetch --file "$lock"
done
