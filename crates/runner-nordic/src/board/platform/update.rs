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

use wasefire_board_api::Supported;
use wasefire_board_api::platform::update::Api;
use wasefire_error::Error;
use wasefire_sync::TakeCell;

use crate::storage::{Storage, StorageWriter};

pub enum Impl {}

impl Supported for Impl {}

impl Api for Impl {
    fn initialize(dry_run: bool) -> Result<(), Error> {
        STATE.with(|state| state.start(dry_run))
    }

    fn process(chunk: &[u8]) -> Result<(), Error> {
        STATE.with(|state| state.write(chunk))
    }

    fn finalize() -> Result<(), Error> {
        STATE.with(|state| {
            let dry_run = state.dry_run()?;
            state.finish()?;
            match dry_run {
                true => Ok(()),
                false => super::reboot(),
            }
        })
    }
}

pub fn init(storage: Storage) {
    STATE.put(StorageWriter::new(storage));
}

static STATE: TakeCell<StorageWriter> = TakeCell::new(None);
