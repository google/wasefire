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

set -ex

# This script setups a GitHub Codespace by downloading and installing the CLI.

curl -fLOSs https://github.com/google/wasefire/releases/latest/download\
/wasefire-x86_64-unknown-linux-gnu.tar.gz

tar xf wasefire-x86_64-unknown-linux-gnu.tar.gz
rm wasefire-x86_64-unknown-linux-gnu.tar.gz
mkdir -p ~/.local/bin
mv wasefire-x86_64-unknown-linux-gnu ~/.local/bin/wasefire

~/.local/bin/wasefire completion bash --output=wasefire
sudo install --mode=644 wasefire /usr/share/bash-completion/completions
rm wasefire

echo 'export WASEFIRE_PROTOCOL=unix' >> ~/.bashrc

sudo apt-get install -y binaryen wabt
