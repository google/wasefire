#!/bin/sh
# Copyright 2024 Google LLC
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

# This script generates release artifacts.

DATE=$(git log -1 --pretty=%cs)
i "Generate notes.txt"
cat <<EOF > notes.txt
See the [changelog] for the list of changes in this release.

You can use the following command to verify a downloaded asset:

    gh attestation verify --repo=google/wasefire <asset-path>

[changelog]: https://github.com/google/wasefire/blob/main/docs/releases/$DATE.md
EOF

i "Generate artifacts and artifacts.txt"
mkdir artifacts

i "Build the CLI for supported targets"
TARGETS='
x86_64-unknown-linux-gnu
'
( cd crates/cli
  for target in $TARGETS; do
    x cargo build --release --target=$target
    cp ../../target/$target/release/wasefire ../../artifacts/wasefire-$target
    echo "artifacts/wasefire-$target#Wasefire CLI ($target)" >> ../../artifacts.txt
  done
)
