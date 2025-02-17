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

//! Provides API for clock.

use wasefire_applet_api::clock as api;
use wasefire_error::Error;

use crate::convert_unit;

/// Returns the time spent since some initial event, in micro-seconds.
///
/// The initial event may be the first time this function is called.
pub fn uptime_us() -> Result<u64, Error> {
    let mut result = 0u64;
    let params = api::uptime_us::Params { ptr: &mut result };
    convert_unit(unsafe { api::uptime_us(params) })?;
    Ok(result)
}
