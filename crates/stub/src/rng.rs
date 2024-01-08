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

use rand::RngCore;
use wasefire_applet_api::rng as api;

#[no_mangle]
unsafe extern "C" fn env_rb(params: api::fill_bytes::Params) -> isize {
    let api::fill_bytes::Params { ptr, len } = params;
    let buf = unsafe { std::slice::from_raw_parts_mut(ptr, len) };
    rand::thread_rng().fill_bytes(buf);
    0
}
