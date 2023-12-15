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

use alloc::string::String;

use wasefire_logger as log;

#[derive(Default)]
pub struct WasefireWrite {
    buf: String,
}

impl core::fmt::Write for WasefireWrite {
    fn write_str(&mut self, data: &str) -> core::fmt::Result {
        self.buf.write_str(data)
    }
}

impl Drop for WasefireWrite {
    fn drop(&mut self) {
        log::debug!(self.buf)
    }
}
