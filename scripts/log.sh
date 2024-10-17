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

x() { ( set -x; "$@" ) }
y() { ( set -x; exec "$@" ) & echo pid=$!; }
i() { _log '1;36' Info "$*"; }
w() { _log '1;33' Warn "$*"; }
t() { _log '1;33' Todo "$*"; }
d() { _log '1;32' Done "$*"; exit 0; }
e() { _log '1;31' Error "$*"; exit 1; }

# We put the escape character in a variable because bash doesn't interpret escaped characters and
# some scripts use bash instead of sh.
_LOG=$(printf '\e')
_log() { echo "$_LOG[$1m$2:$_LOG[m $3"; }
