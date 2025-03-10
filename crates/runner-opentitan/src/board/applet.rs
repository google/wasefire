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

use wasefire_board_api::applet::Api;
use wasefire_error::{Code, Error};

use crate::board::storage::Linear;
use crate::board::with_state;

pub struct State {
    install: Option<Linear>,
}

pub fn init() -> State {
    State { install: None }
}

pub enum Impl {}

impl Api for Impl {
    unsafe fn get() -> Result<&'static [u8], Error> {
        with_state(|state| {
            if state.applet.install.is_some() {
                return Err(Error::user(Code::InvalidState));
            }
            let side = crate::flash::inactive() as usize;
            let storage = unsafe { state.storage.applets[side].read() };
            let length = !u32::from_ne_bytes(storage[storage.len() - 4 ..].try_into().unwrap());
            storage.get(.. length as usize).ok_or(Error::internal(Code::InvalidState))
        })
    }

    fn start(dry_run: bool) -> Result<(), Error> {
        with_state(|state| {
            let side = crate::flash::inactive() as usize;
            let writing = Linear::start(dry_run, &state.storage.applets[side])?;
            state.applet.install = Some(writing);
            Ok(())
        })
    }

    fn write(chunk: &[u8]) -> Result<(), Error> {
        with_state(|state| {
            let Some(writing) = &mut state.applet.install else {
                return Err(Error::user(Code::InvalidState));
            };
            let side = crate::flash::inactive() as usize;
            writing.write(&state.storage.applets[side], chunk)
        })
    }

    fn finish() -> Result<(), Error> {
        with_state(|state| {
            let Some(writing) = state.applet.install.take() else {
                return Err(Error::user(Code::InvalidState));
            };
            let side = crate::flash::inactive() as usize;
            let raw = &state.storage.applets[side];
            if let Some(length) = writing.flush(raw)? {
                let length = !(length as u32);
                let mut chunk = [0xff; Linear::BUFFER_SIZE];
                chunk[Linear::BUFFER_SIZE - 4 ..].copy_from_slice(&length.to_ne_bytes());
                raw.write(raw.len() - chunk.len(), &chunk)?;
            }
            Ok(())
        })
    }
}
