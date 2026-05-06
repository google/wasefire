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

# This script generates local JSON schema store files.

SOURCE=third_party/SchemaStore/schemastore
TARGET=target/tombi/https/www.schemastore.org

[ -e $SOURCE/.git ] || x git submodule update --init $SOURCE

mkdir -p $TARGET/api/json
cp $SOURCE/src/api/json/catalog.json $TARGET/api/json

copy_schema() {
  cp $SOURCE/src/schemas/json/$1.json $TARGET
}

copy_schema cargo
copy_schema cargo-lints-clippy
copy_schema cargo-lints-rust
copy_schema rust-toolchain
copy_schema rustfmt
