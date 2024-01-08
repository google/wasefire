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

use wasefire_board_api::usb::Event;
use wasefire_board_api::Api as Board;

#[cfg(all(feature = "board-api-usb-serial", feature = "applet-api-usb-serial"))]
pub mod serial;

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Key {
    #[cfg(all(feature = "board-api-usb-serial", feature = "applet-api-usb-serial"))]
    Serial(serial::Key),
}

impl<B: Board> From<Key> for crate::event::Key<B> {
    fn from(key: Key) -> Self {
        crate::event::Key::Usb(key)
    }
}

impl<'a> From<&'a Event> for Key {
    fn from(event: &'a Event) -> Self {
        match event {
            #[cfg(all(feature = "board-api-usb-serial", feature = "applet-api-usb-serial"))]
            Event::Serial(event) => Key::Serial(event.into()),
        }
    }
}

pub fn process(event: Event) {
    match event {
        #[cfg(all(feature = "board-api-usb-serial", feature = "applet-api-usb-serial"))]
        Event::Serial(_) => serial::process(),
    }
}
