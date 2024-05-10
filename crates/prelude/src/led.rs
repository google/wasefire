// Copyright 2022 Google LLC
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

//! Provides API for LEDs.
//!
//! LEDs are abstracted with:
//! - They have 2 states: `On` or `Off`.
//! - Their state can be read and written.

use wasefire_applet_api::led as api;

pub use self::api::Status;
pub use self::api::Status::*;
use crate::{convert, convert_bool, convert_unit};

/// Returns the number of available LEDs on the board.
pub fn count() -> usize {
    convert(unsafe { api::count() }).unwrap_or(0)
}

/// Returns the status of a LED.
///
/// The `led` argument is the index of the LED. It must be less than [count()].
#[track_caller]
pub fn get(led: usize) -> api::Status {
    let status = convert_bool(unsafe { api::get(api::get::Params { led }) }).unwrap();
    match status {
        false => api::Status::Off,
        true => api::Status::On,
    }
}

/// Sets the status of a LED.
///
/// The `led` argument is the index of the LED. It must be less than [count()]. The `status`
/// argument is the new status.
#[track_caller]
pub fn set(led: usize, status: api::Status) {
    let params = api::set::Params { led, status: status as usize };
    convert_unit(unsafe { api::set(params) }).unwrap();
}
