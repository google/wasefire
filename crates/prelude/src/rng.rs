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

//! Provides API for random number generation.

use wasefire_applet_api::rng as api;

/// Fills a slice with random bytes.
pub fn fill_bytes(buf: &mut [u8]) {
    let params = api::fill_bytes::Params { ptr: buf.as_mut_ptr(), len: buf.len() };
    unsafe { api::fill_bytes(params) }
}
