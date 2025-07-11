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

use wasefire_board_api::Supported;
use wasefire_board_api::platform::update::Api;
use wasefire_error::{Code, Error};
use wasefire_logger as log;

use crate::board::platform::version;
use crate::board::storage::Linear;
use crate::board::with_state;

pub struct State {
    storage: Option<Linear>,
}

pub fn init() -> State {
    State { storage: None }
}

pub enum Impl {}

impl Supported for Impl {}

impl Api for Impl {
    fn initialize(dry_run: bool) -> Result<(), Error> {
        with_state(|state| {
            let writing = Linear::start(dry_run, &state.storage.other)?;
            state.platform.update.storage = Some(writing);
            Ok(())
        })
    }

    fn process(chunk: &[u8]) -> Result<(), Error> {
        with_state(|state| {
            let Some(writing) = &mut state.platform.update.storage else {
                return Err(Error::user(Code::InvalidState));
            };
            writing.write(&state.storage.other, chunk)
        })
    }

    fn finalize() -> Result<(), Error> {
        with_state(|state| {
            let Some(writing) = state.platform.update.storage.take() else {
                return Err(Error::user(Code::InvalidState));
            };
            if writing.flush(&state.storage.other)?.is_some() {
                reboot()?;
            }
            Ok(())
        })
    }
}

fn reboot() -> Result<!, Error> {
    let next = crate::flash::active().opposite();
    let primary = if version(true) < version(false) { next.opposite() } else { next };
    log::info!("Rebooting to side {} (primary={})", next, primary);
    crate::bootsvc::next_boot(Some(next), Some(primary))?;
    crate::reboot();
}
