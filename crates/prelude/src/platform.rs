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

//! Provides API to interact with the platform.

use wasefire_applet_api::platform as api;

pub mod update;

/// Reboots the device (thus platform and applets).
pub fn reboot() -> ! {
    let api::reboot::Results { res } = unsafe { api::reboot() };
    unreachable!("reboot() returned {res}");
}
