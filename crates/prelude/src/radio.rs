// Copyright 2023 Google LLC
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

//! Provides API for radio.

/// Errors returned by radio operations.
pub use wasefire_applet_api::radio::Error;

pub mod ble;

fn convert(len: isize) -> Result<usize, Error> {
    if len < 0 {
        return Err(Error::Unknown);
    }
    Ok(len as usize)
}
