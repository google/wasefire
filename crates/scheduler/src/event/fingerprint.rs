// Copyright 2025 Google LLC
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

use alloc::vec::Vec;

use wasefire_board_api::Api as Board;
use wasefire_board_api::fingerprint::Event;
use wasefire_error::Error;

use crate::applet::Applet;

#[cfg(feature = "board-api-fingerprint-matcher")]
pub mod matcher;
#[cfg(feature = "board-api-fingerprint-sensor")]
pub mod sensor;

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Key {
    #[cfg(feature = "board-api-fingerprint-matcher")]
    Matcher(matcher::Key),

    #[cfg(feature = "board-api-fingerprint-sensor")]
    Sensor(sensor::Key),
}

impl<B: Board> From<Key> for crate::event::Key<B> {
    fn from(key: Key) -> Self {
        crate::event::Key::Fingerprint(key)
    }
}

impl<'a> From<&'a Event> for Key {
    fn from(event: &'a Event) -> Self {
        match event {
            #[cfg(feature = "board-api-fingerprint-matcher")]
            Event::Matcher(event) => Key::Matcher(event.into()),
            #[cfg(feature = "board-api-fingerprint-sensor")]
            Event::Sensor(event) => Key::Sensor(event.into()),
        }
    }
}

impl Key {
    pub fn disable<B: Board>(self) -> Result<(), Error> {
        match self {
            #[cfg(feature = "board-api-fingerprint-matcher")]
            Key::Matcher(x) => x.disable::<B>(),
            #[cfg(feature = "board-api-fingerprint-sensor")]
            Key::Sensor(x) => x.disable::<B>(),
        }
    }
}

pub fn process<B: Board>(event: Event, params: &mut Vec<u32>, applet: &mut Applet<B>) {
    match event {
        #[cfg(feature = "board-api-fingerprint-matcher")]
        Event::Matcher(event) => matcher::process(event, params, applet),
        #[cfg(feature = "board-api-fingerprint-sensor")]
        Event::Sensor(event) => sensor::process(event, params, applet),
    }
}
