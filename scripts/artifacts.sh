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
. scripts/system.sh
. scripts/test-helper.sh

# This script generates release artifacts.

DATE=$(git log -1 --pretty=%cs)
TOOLCHAIN=$(sed -n 's/"/`/g;s/^channel = //p' rust-toolchain.toml)
i "Generate notes.txt"
cat <<EOF > notes.txt
See the [changelog] for the list of changes in this release.

This release was built with the $TOOLCHAIN toolchain. Applets may need to be built with the same
toolchain. The CLI will install and use this toolchain if \`rustup\` is installed.

You can use the CLI or the [Web UI] to update your platform using an appropriate "Wasefire platform
update" file. Otherwise, you can flash a platform from scratch using an appropriate "Wasefire
platform bootstrap" file.

You can use the following command to check your downloaded assets:

    sha256sum --ignore-missing --check sha256sum.txt

You can use one of the following commands to verify a downloaded asset:

    gh attestation verify --repo=google/wasefire ASSET_PATH
    gh attestation verify --owner=google --bundle=attestation.intoto.jsonl ASSET_PATH

[Web UI]: https://google.github.io/wasefire/webui
[changelog]: https://github.com/google/wasefire/blob/main/docs/releases/$DATE.md
EOF

x mkdir artifacts

cp_artifact() {
  [ -z "$1" ] || x cp "$1" "artifacts/$2"
  echo "artifacts/$2#$3" >> artifacts.txt
}

i "Build web-client once for all supported targets"
( cd crates/runner-host/crates/web-client && make )

i "Build the CLI for each supported target"
TARGETS='
i686-unknown-linux-gnu
x86_64-unknown-linux-gnu
'
for target in $TARGETS; do
  ( set -x
    if [ $target = i686-unknown-linux-gnu ]; then
      ensure apt libusb-1.0-0:i386
      export PKG_CONFIG_LIBDIR=/usr/lib/i386-linux-gnu/pkgconfig
      export PKG_CONFIG_SYSROOT_DIR=/usr/lib/i386-linux-gnu/pkgconfig
    fi
    cargo build --manifest-path=crates/runner-host/Cargo.toml --release --target=$target \
      --features=debug,wasm
    export WASEFIRE_HOST_PLATFORM=$PWD/target/$target/release/runner-host
    cargo build --manifest-path=crates/cli/Cargo.toml --release --target=$target --features=_prod
    cp target/$target/release/wasefire artifacts/wasefire-$target
    cd artifacts
    tar czf wasefire-$target.tar.gz wasefire-$target
    rm wasefire-$target
  )
  cp_artifact '' wasefire-$target.tar.gz "Wasefire CLI ($target)"
done

# We increment the version at each release.
VERSION=0001
FLAGS="--name=github-$DATE --version=$VERSION"
i "Build a simple Nordic platform for each supported board"
yes | x cargo xtask --release runner nordic $FLAGS flash --artifacts
cp_artifact target/wasefire/platform.hex platform-nordic-devkit.hex \
  'Wasefire platform bootstrap (nRF52840 DK)'
x cargo xtask --release runner nordic $FLAGS bundle
cp_artifact wasefire/platform.wfb platform-nordic-devkit.wfb \
  'Wasefire platform update (nRF52840 DK)'
yes | x cargo xtask --release runner nordic --board=dongle $FLAGS flash --artifacts
cp_artifact target/wasefire/platform.hex platform-nordic-dongle-1.hex \
  'Wasefire platform bootstrap (nRF52840 Dongle) step 1'
cp_artifact target/wasefire/bootloader.hex platform-nordic-dongle-2.hex \
  'Wasefire platform bootstrap (nRF52840 Dongle) step 2'
x cargo xtask --release runner nordic --board=dongle $FLAGS bundle
cp_artifact wasefire/platform.wfb platform-nordic-dongle.wfb \
  'Wasefire platform update (nRF52840 Dongle)'
yes | x cargo xtask --release runner nordic --board=makerdiary $FLAGS flash --artifacts
cp_artifact target/wasefire/platform.hex platform-nordic-makerdiary.hex \
  'Wasefire platform bootstrap (nRF52840 MDK USB Dongle)'
x cargo xtask --release runner nordic --board=makerdiary $FLAGS bundle
cp_artifact wasefire/platform.wfb platform-nordic-makerdiary.wfb \
  'Wasefire platform update (nRF52840 MDK USB Dongle)'
