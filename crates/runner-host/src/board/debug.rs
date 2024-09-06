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

use std::sync::OnceLock;
use std::time::Instant;

use wasefire_board_api as board;

pub enum Impl {}

impl board::debug::Api for Impl {
    const MAX_TIME: u64 = u64::MAX;

    #[cfg(feature = "web")]
    fn println(line: &str) {
        let time = Self::time();
        let message = format!("{}.{:06}: {}", time / 1000000, time % 1000000, line);
        crate::with_state(|state| state.web.println(message))
    }

    fn time() -> u64 {
        static ORIGIN: OnceLock<Instant> = OnceLock::new();
        let now = Instant::now();
        let origin = ORIGIN.get_or_init(|| now);
        now.duration_since(*origin).as_micros() as u64
    }

    fn exit(success: bool) -> ! {
        crate::cleanup::shutdown(if success { 0 } else { 1 })
    }
}
