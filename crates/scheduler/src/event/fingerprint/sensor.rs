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

use wasefire_board_api::fingerprint::sensor::{Api as _, Event};
use wasefire_board_api::{self as board, Api as Board};
use wasefire_error::{Code, Error};

use crate::applet::Applet;
use crate::applet::store::MemoryApi;

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Key {
    Capture,
}

impl<B: Board> From<Key> for crate::event::Key<B> {
    fn from(key: Key) -> Self {
        super::Key::Sensor(key).into()
    }
}

impl<'a> From<&'a Event> for Key {
    fn from(event: &'a Event) -> Self {
        match event {
            Event::CaptureDone | Event::CaptureError { .. } => Key::Capture,
        }
    }
}

impl Key {
    pub fn disable<B: Board>(self) -> Result<(), Error> {
        use wasefire_board_api::fingerprint::sensor::Api as _;
        match self {
            Key::Capture => board::fingerprint::Sensor::<B>::abort_capture(),
        }
    }
}

pub fn process<B: Board>(event: Event, params: &mut Vec<u32>, applet: &mut Applet<B>) {
    match event {
        Event::CaptureDone => {
            let width = board::fingerprint::Sensor::<B>::IMAGE_WIDTH as u32;
            let height = board::fingerprint::Sensor::<B>::IMAGE_HEIGHT as u32;
            let len = width * height;
            let mut memory = applet.memory();
            let image: Result<_, _> = try {
                let ptr = memory.alloc(len, 1).map_err(|_| Error::user(Code::NotEnough))?;
                let dst = memory.get_mut(ptr, len).map_err(|_| Error::user(Code::OutOfBounds))?;
                board::fingerprint::Sensor::<B>::read_capture(dst)?;
                ptr
            };
            params.push(image.unwrap_or(0));
            params.push(Error::encode(image.map(|_| width)) as u32);
            params.push(height);
        }
        Event::CaptureError { error } => {
            params.push(0);
            params.push(Error::encode(Err(error)) as u32);
            params.push(0);
        }
    }
    applet.disable_noerror(Key::Capture.into());
}
