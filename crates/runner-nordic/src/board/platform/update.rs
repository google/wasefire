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

use alloc::boxed::Box;
use alloc::vec::Vec;

use header::{Header, Side};
use wasefire_board_api::platform::update::Api;
use wasefire_board_api::Supported;
use wasefire_error::{Code, Error};
use wasefire_logger as log;
use wasefire_sync::TakeCell;

use crate::storage::{Storage, StorageWriter};

pub enum Impl {}

impl Supported for Impl {}

impl Api for Impl {
    fn metadata() -> Result<Box<[u8]>, Error> {
        let mut metadata = Vec::new();
        let side = Side::current().ok_or(Error::world(Code::InvalidState))?;
        push_header(&mut metadata, Header::new(side));
        push_header(&mut metadata, Header::new(!side));
        Ok(metadata.into_boxed_slice())
    }

    fn initialize(dry_run: bool) -> Result<(), Error> {
        STATE.with(|state| state.start(dry_run))
    }

    fn process(chunk: &[u8]) -> Result<(), Error> {
        STATE.with(|state| state.write(chunk))
    }

    fn finalize() -> Result<(), Error> {
        STATE.with(|state| {
            let dry_run = state.dry_run()?;
            state.flush()?;
            match dry_run {
                true => Ok(()),
                false => super::reboot(),
            }
        })
    }
}

pub fn init(storage: Storage) {
    match StorageWriter::new(storage) {
        Ok(x) => STATE.put(x),
        Err((s, e)) => {
            log::error!("Init storage [{:08x}; {:08x}] failed: {}", s.ptr(), s.len(), e);
        }
    }
}

static STATE: TakeCell<StorageWriter> = TakeCell::new(None);

fn push_header(metadata: &mut Vec<u8>, header: Header) {
    match header.side() {
        Side::A => metadata.push(0xa),
        Side::B => metadata.push(0xb),
    }
    for i in 0 .. 3 {
        metadata.push(0xff * header.attempt(i).free() as u8);
    }
    metadata.extend_from_slice(&header.timestamp().to_be_bytes());
}
