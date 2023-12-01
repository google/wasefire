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
// limitations under the License.use opensk_lib::api::clock::Clock;

use opensk_lib::api::clock::Clock;

#[derive(Default)]
pub struct WasefireClock {}

impl Clock for WasefireClock {
    type Timer = Self;

    fn make_timer(&mut self, milliseconds: usize) -> Self::Timer {
        todo!()
    }

    fn is_elapsed(&mut self, timer: &Self::Timer) -> bool {
        todo!()
    }
}
