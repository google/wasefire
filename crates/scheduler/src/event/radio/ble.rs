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

use wasefire_board_api::radio::ble::Event;
use wasefire_board_api::{self as board, Api as Board};
use wasefire_error::Error;

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Key {
    Advertisement,
}

impl<B: Board> From<Key> for crate::event::Key<B> {
    fn from(key: Key) -> Self {
        super::Key::Ble(key).into()
    }
}

impl<'a> From<&'a Event> for Key {
    fn from(event: &'a Event) -> Self {
        match event {
            Event::Advertisement => Key::Advertisement,
        }
    }
}

impl Key {
    pub fn disable<B: Board>(self) -> Result<(), Error> {
        use wasefire_board_api::radio::ble::Api as _;
        board::radio::Ble::<B>::disable(&Event::Advertisement)
    }
}

pub fn process() {}
