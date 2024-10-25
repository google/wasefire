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

use derive_where::derive_where;
use wasefire_board_api::timer::Event;
use wasefire_board_api::{self as board, Api as Board, Id};
use wasefire_error::Error;

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive_where(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Key<B: Board> {
    pub timer: Id<board::Timer<B>>,
}

impl<B: Board> From<Key<B>> for crate::event::Key<B> {
    fn from(key: Key<B>) -> Self {
        crate::event::Key::Timer(key)
    }
}

impl<'a, B: Board> From<&'a Event<B>> for Key<B> {
    fn from(event: &'a Event<B>) -> Self {
        Key { timer: event.timer }
    }
}

impl<B: Board> Key<B> {
    pub fn disable(self) -> Result<(), Error> {
        use wasefire_board_api::timer::Api as _;
        board::Timer::<B>::disarm(self.timer)
    }
}

pub fn process() {}
