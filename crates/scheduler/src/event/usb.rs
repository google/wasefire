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

use wasefire_board_api::Api as Board;
use wasefire_board_api::usb::Event;
use wasefire_error::Error;

#[cfg(feature = "board-api-usb-ctap")]
pub mod ctap;
#[cfg(feature = "board-api-usb-serial")]
pub mod serial;

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Key {
    #[cfg(feature = "board-api-usb-ctap")]
    Ctap(ctap::Key),

    #[cfg(feature = "board-api-usb-serial")]
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
            #[cfg(feature = "board-api-usb-ctap")]
            Event::Ctap(event) => Key::Ctap(event.into()),
            #[cfg(feature = "board-api-usb-serial")]
            Event::Serial(event) => Key::Serial(event.into()),
        }
    }
}

impl Key {
    pub fn disable<B: Board>(self) -> Result<(), Error> {
        match self {
            #[cfg(feature = "board-api-usb-ctap")]
            Key::Ctap(x) => x.disable::<B>(),
            #[cfg(feature = "board-api-usb-serial")]
            Key::Serial(x) => x.disable::<B>(),
        }
    }
}

pub fn process(event: Event) {
    match event {
        #[cfg(feature = "board-api-usb-ctap")]
        Event::Ctap(_) => ctap::process(),
        #[cfg(feature = "board-api-usb-serial")]
        Event::Serial(_) => serial::process(),
    }
}
