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
TARGET=target/schemastore

[ -e $SOURCE/.git ] || x git submodule update --init $SOURCE
[ -e $TARGET ] || x mkdir -p $TARGET

convert() {
  PATTERN='\(Cargo\|rustfmt\|rust-toolchain\).toml'
  sed -n '/^    /!p;/^    {/{:a;N;/\n    },\?$/!ba;/'"$PATTERN"'/p}' \
    | sed '/"url"/s#https://www.schemastore.org#file://'"$PWD/$TARGET"'#' \
    | sed '/^    },$/{N;s/},\n  ]/}\n  ]/}'
}

convert < $SOURCE/src/api/json/catalog.json > $TARGET/catalog.json
for file in $(sed -n 's#^.*/\([^/]*\.json\)"$#\1#p' $TARGET/catalog.json); do
  cp $SOURCE/src/schemas/json/$file $TARGET
done
