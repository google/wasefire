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

shard_init() {
  _SHARD_INDEX=${1:-0}
  _SHARD_COUNT=${2:-1}
  echo "Initialize sharding for $_SHARD_INDEX modulo $_SHARD_COUNT"
  _shard_index=-1
}

shard_next() {
  _shard_index=$(( _shard_index + 1 ))
  [ $(( _shard_index % _SHARD_COUNT )) -eq $_SHARD_INDEX ] || return
  echo "Current shard is $_shard_index"
}
