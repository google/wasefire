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

use wasefire_board_api::fingerprint::matcher::{Api as _, Event};
use wasefire_board_api::{self as board, Api as Board, AppletMemory as _};
use wasefire_error::{Code, Error};

use crate::applet::Applet;

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Key {
    Enroll,
    EnrollStep,
    Identify,
}

impl<B: Board> From<Key> for crate::event::Key<B> {
    fn from(key: Key) -> Self {
        super::Key::Matcher(key).into()
    }
}

impl<'a> From<&'a Event> for Key {
    fn from(event: &'a Event) -> Self {
        match event {
            Event::EnrollStep { .. } => Key::EnrollStep,
            Event::EnrollDone | Event::EnrollError { .. } => Key::Enroll,
            Event::IdentifyDone { .. } | Event::IdentifyError { .. } => Key::Identify,
        }
    }
}

impl Key {
    pub fn disable<B: Board>(self) -> Result<(), Error> {
        use wasefire_board_api::fingerprint::matcher::Api as _;
        match self {
            Key::Enroll => board::fingerprint::Matcher::<B>::abort_enroll(),
            Key::EnrollStep => Ok(()),
            Key::Identify => board::fingerprint::Matcher::<B>::abort_identify(),
        }
    }
}

pub fn process<B: Board>(event: Event, params: &mut Vec<u32>, applet: &mut Applet<B>) {
    match event {
        Event::EnrollStep { remaining } => params.push(remaining as u32),
        Event::EnrollDone => {
            read_template(params, applet, 0, board::fingerprint::Matcher::<B>::read_enroll);
        }
        Event::IdentifyDone { result } => {
            if result {
                read_template(params, applet, 1, board::fingerprint::Matcher::<B>::read_identify);
            } else {
                params.extend_from_slice(&[0, 0]);
            }
        }
        Event::EnrollError { error } | Event::IdentifyError { error } => {
            params.push(Error::encode(Err(error)) as u32);
            params.push(0);
        }
    }
    match event {
        Event::EnrollStep { .. } => (),
        Event::EnrollDone | Event::EnrollError { .. } => {
            applet.disable_noerror(Key::Enroll.into());
            applet.disable_noerror(Key::EnrollStep.into());
        }
        Event::IdentifyDone { .. } | Event::IdentifyError { .. } => {
            applet.disable_noerror(Key::Identify.into());
        }
    }
}

fn read_template<B: Board>(
    params: &mut Vec<u32>, applet: &mut Applet<B>, ok: u32,
    read: fn(&mut [u8]) -> Result<(), Error>,
) {
    let len = board::fingerprint::Matcher::<B>::TEMPLATE_ID_SIZE as u32;
    let mut memory = applet.memory();
    let template: Result<_, _> = try {
        let ptr = memory.alloc(len, 1).map_err(|_| Error::user(Code::NotEnough))?;
        let dst = memory.get_mut(ptr, len).map_err(|_| Error::user(Code::OutOfBounds))?;
        read(dst)?;
        ptr
    };
    params.push(Error::encode(template.map(|_| ok)) as u32);
    params.push(template.unwrap_or(0));
}
