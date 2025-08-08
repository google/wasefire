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

pub mod install;

pub enum Impl {}

impl Api for Impl {
    type Install = install::Impl;

    unsafe fn get() -> Result<&'static [u8], Error> {
        STATE.with(|state| {
            let storage = unsafe { state.writer.get() }?;
            Ok(&storage[.. state.size])
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
    let page_size = storage.page_size();
    StorageIndex { page: num_pages - 1, byte: page_size - 4 }
}

fn read_size(storage: &Storage) -> Result<usize, Error> {
    let word = storage.read_slice(last_word(storage), 4)?;
    Ok(!usize::from_ne_bytes(*<&[u8; 4]>::try_from(&word[..]).unwrap()))
}

fn write_size(storage: &mut Storage, size: usize) -> Result<(), Error> {
    storage.write_slice(last_word(storage), &(!size).to_ne_bytes())
}
