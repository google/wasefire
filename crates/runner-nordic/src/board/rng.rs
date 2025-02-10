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

use wasefire_board_api::Error;
use wasefire_board_api::rng::Api;

use crate::with_state;

pub enum Impl {}

impl Api for Impl {
    fn fill_bytes(buffer: &mut [u8]) -> Result<(), Error> {
        with_state(|state| state.rng.random(buffer));
        Ok(())
    }
}
