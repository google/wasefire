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

use std::sync::OnceLock;
use std::time::Instant;

use wasefire_board_api::Error;
use wasefire_board_api::clock::Api;

pub fn init() {
    let _ = Impl::uptime_us();
}

pub fn uptime_us() -> u64 {
    let now = Instant::now();
    let origin = ORIGIN.get_or_init(|| now);
    now.duration_since(*origin).as_micros() as u64
}

pub enum Impl {}

impl Api for Impl {
    fn uptime_us() -> Result<u64, Error> {
        Ok(uptime_us())
    }
}

static ORIGIN: OnceLock<Instant> = OnceLock::new();
