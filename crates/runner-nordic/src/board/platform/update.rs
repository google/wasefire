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
use wasefire_store::{Storage as _, StorageError, StorageIndex};
use wasefire_sync::TakeCell;

use crate::storage::Storage;

pub enum Impl {}

impl Supported for Impl {}

impl Api for Impl {
    fn metadata() -> Result<Box<[u8]>, usize> {
        let mut metadata = Vec::new();
        let side = Side::current().ok_or(Error::NoSide)?;
        push_header(&mut metadata, Header::new(side));
        push_header(&mut metadata, Header::new(!side));
        Ok(metadata.into_boxed_slice())
    }

    fn initialize(dry_run: bool) -> Result<(), usize> {
        STATE.with(|state| {
            state.reset(dry_run);
            Ok(())
        })
    }

    fn process(mut chunk: &[u8]) -> Result<(), usize> {
        STATE.with(|state| {
            while !chunk.is_empty() {
                state.write(&mut chunk)?;
            }
            Ok(())
        })
    }

    fn finalize() -> Result<(), usize> {
        STATE.with(|state| {
            state.flush()?;
            match state.dry_run {
                true => Ok(()),
                false => super::reboot(),
            }
        })
    }
}

pub fn init(storage: Storage) {
    STATE.put(Update::new(storage));
}

static STATE: TakeCell<Update> = TakeCell::new(None);

struct Update {
    storage: Storage,
    dry_run: bool,
    // offset + buffer.len() <= storage.len()
    offset: usize,
    buffer: Vec<u8>,
}

impl Update {
    fn new(storage: Storage) -> Self {
        Update { storage, dry_run: false, offset: 0, buffer: Vec::new() }
    }

    fn reset(&mut self, dry_run: bool) {
        self.dry_run = dry_run;
        self.offset = 0;
        self.buffer.clear();
    }

    fn write(&mut self, chunk: &mut &[u8]) -> Result<(), Error> {
        let page_size = self.storage.page_size();
        let word_size = self.storage.word_size();
        let page = self.offset / page_size;
        let byte = self.offset % page_size;
        let index = StorageIndex { page, byte };
        let (value, rest) = if self.buffer.is_empty() {
            let length = core::cmp::min(chunk.len(), page_size - byte);
            let length = length / word_size * word_size;
            chunk.split_at(length)
        } else {
            assert!(self.buffer.len() < word_size);
            let length = core::cmp::min(chunk.len(), word_size - self.buffer.len());
            self.buffer.extend_from_slice(&chunk[.. length]);
            (&self.buffer[..], &chunk[length ..])
        };
        if value.is_empty() {
            self.buffer = chunk.to_vec();
            *chunk = &[];
            return Ok(());
        }
        if !self.dry_run {
            if byte == 0 {
                self.storage.erase_page(page)?;
            }
            self.storage.write_slice(index, value)?;
        }
        *chunk = rest;
        self.offset += value.len();
        Ok(())
    }

    fn flush(&mut self) -> Result<(), Error> {
        if self.buffer.is_empty() {
            return Ok(());
        }
        let word_size = self.storage.word_size();
        assert!(self.buffer.len() < word_size);
        self.buffer.resize(word_size, 0xff);
        let chunk = core::mem::take(&mut self.buffer);
        let mut chunk = &chunk[..];
        self.write(&mut chunk)?;
        assert!(chunk.is_empty());
        Ok(())
    }
}

#[repr(usize)]
enum Error {
    _Unknown = 0x00,
    // Internal errors (0x01 ..= 0x7f).
    NoSide = 0x01,
    Storage = 0x02,
    // User errors (0x81 ..= 0xff).
    OutOfBounds = 0x81,
    NotAligned = 0x82,
}

impl From<StorageError> for Error {
    fn from(value: StorageError) -> Self {
        match value {
            StorageError::NotAligned => Error::NotAligned,
            StorageError::OutOfBounds => Error::OutOfBounds,
            StorageError::CustomError => Error::Storage,
        }
    }
}

impl From<Error> for usize {
    fn from(value: Error) -> Self {
        value as usize
    }
}

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
