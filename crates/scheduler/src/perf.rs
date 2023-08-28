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

use core::marker::PhantomData;

use bytemuck::Zeroable;
use wasefire_applet_api::debug as api;
use wasefire_board_api::debug::Api as _;
use wasefire_board_api::{self as board, Api as Board};

pub struct Perf<B: Board> {
    start: u64,
    value: api::Perf,
    board: PhantomData<B>,
}

impl<B: Board> Default for Perf<B> {
    fn default() -> Self {
        Perf { start: board::Debug::<B>::time(), value: api::Perf::zeroed(), board: PhantomData }
    }
}

impl<B: Board> Perf<B> {
    pub fn record(&mut self, slot: Slot) {
        let end = board::Debug::<B>::time();
        let start = core::mem::replace(&mut self.start, end);
        let value = match slot {
            Slot::Platform => &mut self.value.platform,
            Slot::Applets => &mut self.value.applets,
            Slot::Waiting => &mut self.value.waiting,
        };
        *value +=
            if end < start { board::Debug::<B>::MAX_TIME - start + end + 1 } else { end - start };
    }

    pub fn read(&mut self) -> api::Perf {
        self.value
    }
}

#[derive(Debug)]
pub enum Slot {
    Platform,
    Applets,
    Waiting,
}
