// Copyright 2024 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::error::Error;

/// Checks the root cause of an error.
///
/// Returns whether the root cause of the error is of the provided type and satisfies the predicate.
pub fn root_cause_is<E: Error + Send + Sync + 'static>(
    error: &anyhow::Error, predicate: impl FnOnce(&E) -> bool,
) -> bool {
    error.root_cause().downcast_ref::<E>().is_some_and(predicate)
}
