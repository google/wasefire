// Copyright 2025 Google LLC
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

//! Provides vendor API.

use wasefire_applet_api::vendor as api;
use wasefire_error::Error;

use crate::convert;

/// Vendor syscalls.
///
/// Those calls are directly forwarded to the board by the scheduler.
///
/// # Safety
///
/// For the syscalls they support, boards must either provide safe libraries or safety documentation
/// (these requirements are not exclusive). If no safety documentation is provided, it must be
/// assumed that this function cannot be called (regardless of its arguments).
pub unsafe fn syscall(x1: usize, x2: usize, x3: usize, x4: usize) -> Result<usize, Error> {
    let params = api::syscall::Params { x1, x2, x3, x4 };
    convert(unsafe { api::syscall(params) })
}
