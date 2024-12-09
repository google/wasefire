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

use wasefire_board_api::Error;
use wasefire_board_api::applet::Api;
use wasefire_store::{Storage as _, StorageIndex};
use wasefire_sync::TakeCell;

use crate::storage::{Storage, StorageWriter};

pub enum Impl {}

impl Api for Impl {
    unsafe fn get() -> Result<&'static [u8], Error> {
        STATE.with(|state| {
            let storage = unsafe { state.writer.get() }?;
            Ok(&storage[.. state.size])
        })
    }

    fn start(dry_run: bool) -> Result<(), Error> {
        STATE.with(|state| {
            state.size = 0;
            state.writer.start(dry_run)
        })
    }

    fn write(chunk: &[u8]) -> Result<(), Error> {
        STATE.with(|state| {
            state.writer.write(chunk)?;
            state.size += chunk.len();
            Ok(())
        })
    }

    fn finish() -> Result<(), Error> {
        STATE.with(|state| {
            let dry_run = state.writer.dry_run()?;
            state.writer.finish()?;
            if dry_run {
                Ok(state.size = read_size(state.writer.storage())?)
            } else {
                write_size(state.writer.storage_mut(), state.size)
            }
        })
    }
}

pub fn init(storage: Storage) {
    let size = read_size(&storage).unwrap_or(0);
    let state = State { writer: StorageWriter::new(storage), size };
    STATE.put(state);
}

static STATE: TakeCell<State> = TakeCell::new(None);

struct State {
    writer: StorageWriter,
    // In sync with the complement of the last 4 bytes of the storage (native-endian) outside a
    // write process.
    size: usize,
}

fn last_word(storage: &Storage) -> StorageIndex {
    let num_pages = storage.num_pages();
    let word_size = storage.word_size();
    let page_size = storage.page_size();
    StorageIndex { page: num_pages - 1, byte: page_size - word_size }
}

fn read_size(storage: &Storage) -> Result<usize, Error> {
    let word_size = storage.word_size();
    let slice = storage.read_slice(last_word(storage), word_size)?;
    Ok(!usize::from_ne_bytes(*<&[u8; 4]>::try_from(&slice[..]).unwrap()))
}

fn write_size(storage: &mut Storage, size: usize) -> Result<(), Error> {
    storage.write_slice(last_word(storage), &(!size).to_ne_bytes())
}
