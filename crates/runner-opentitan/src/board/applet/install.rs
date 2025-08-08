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

use wasefire_board_api::transfer::Api;
use wasefire_error::{Code, Error};

use crate::board::storage::Linear;
use crate::board::with_state;
use crate::flash::PAGE_SIZE;

pub struct State {
    storage: Option<Linear>,
}

impl State {
    pub fn ongoing(&self) -> bool {
        self.storage.is_some()
    }
}

pub fn init() -> State {
    State { storage: None }
}

pub enum Impl {}

impl Api for Impl {
    const CHUNK_SIZE: usize = PAGE_SIZE;

    fn start(dry_run: bool) -> Result<usize, Error> {
        with_state(|state| {
            let side = crate::flash::inactive() as usize;
            let raw = &state.storage.applets[side];
            if !dry_run {
                raw.erase(raw.len() - PAGE_SIZE)?;
            }
            state.applet.install.storage = Some(Linear::start(dry_run)?);
            Ok(raw.len() / PAGE_SIZE)
        })
    }

    fn erase() -> Result<(), Error> {
        with_state(|state| {
            let Some(linear) = &mut state.applet.install.storage else {
                return Err(Error::user(Code::InvalidState));
            };
            let side = crate::flash::inactive() as usize;
            linear.erase(&state.storage.applets[side])
        })
    }

    fn write(chunk: &[u8]) -> Result<(), Error> {
        with_state(|state| {
            let Some(linear) = &mut state.applet.install.storage else {
                return Err(Error::user(Code::InvalidState));
            };
            let side = crate::flash::inactive() as usize;
            linear.write(&state.storage.applets[side], chunk)
        })
    }

    fn finish() -> Result<(), Error> {
        with_state(|state| {
            let Some(linear) = state.applet.install.storage.take() else {
                return Err(Error::user(Code::InvalidState));
            };
            let side = crate::flash::inactive() as usize;
            let raw = &state.storage.applets[side];
            if let Some(length) = linear.finish()? {
                let length = !(length as u32);
                let mut chunk = [0xff; 8];
                chunk[4 ..].copy_from_slice(&length.to_ne_bytes());
                raw.write(raw.len() - chunk.len(), &chunk)?;
            }
            Ok(())
        })
    }
}
