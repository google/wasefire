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

use wasefire_board_api::Api as Board;
use wasefire_board_api::platform::Event;
use wasefire_error::Error;

pub mod protocol;

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Key {
    Protocol(protocol::Key),
}

impl<B: Board> From<Key> for crate::event::Key<B> {
    fn from(key: Key) -> Self {
        crate::event::Key::Platform(key)
    }
}

impl<'a> From<&'a Event> for Key {
    fn from(event: &'a Event) -> Self {
        match *event {
            Event::Protocol(ref event) => Key::Protocol(event.into()),
        }
    }
}

impl Key {
    pub fn disable(self) -> Result<(), Error> {
        match self {
            Key::Protocol(x) => x.disable(),
        }
    }
}

pub fn process(event: Event) {
    match event {
        Event::Protocol(_) => protocol::process(),
    }
}
