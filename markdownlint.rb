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

all

rule 'MD013', :line_length => 100 # Line length
rule 'MD026', :punctuation => '.,;!' # Trailing punctuation in header

exclude_rule 'MD001' # Header levels should only increment by one level at a time
exclude_rule 'MD002' # First header should be a top level header
exclude_rule 'MD024' # Multiple headers with the same content
exclude_rule 'MD041' # First line in file should be a top level header
