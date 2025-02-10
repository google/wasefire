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
// limitations under the License.use opensk_lib::api::clock::Clock;

use opensk_lib::api::clock::Clock;

use crate::env::WasefireEnv;

impl Clock for WasefireEnv {
    type Timer = u64;

    fn make_timer(&mut self, ms: usize) -> Self::Timer {
        now() + ms as u64
    }

    fn is_elapsed(&mut self, timer: &Self::Timer) -> bool {
        now() <= *timer
    }
}

fn now() -> u64 {
    wasefire::debug::time().unwrap()
}
