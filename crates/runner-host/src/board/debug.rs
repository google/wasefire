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

use {wasefire_board_api as board, wasefire_logger as log};

pub enum Impl {}

impl board::debug::Api for Impl {
    const MAX_TIME: u64 = u64::MAX;

    fn println(line: &str) {
        let time = Self::time();
        let timestamp = format!("{}.{:06}", time / 1000000, time % 1000000);
        crate::with_state(|state| match &state.web {
            Some(x) => x.println(timestamp, line.to_string()),
            None => log::println!("{timestamp}: {line}"),
        })
    }

    fn time() -> u64 {
        crate::board::clock::uptime_us()
    }
}
