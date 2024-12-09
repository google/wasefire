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

use core::ffi::{CStr, c_char};

use wasefire_logger as log;

#[unsafe(no_mangle)]
#[linkage = "weak"]
pub(crate) unsafe extern "C" fn env_dispatch(link: *const c_char, _params: *const u32) -> isize {
    log::panic!("{:?} is not defined", unsafe { CStr::from_ptr(link) });
}
