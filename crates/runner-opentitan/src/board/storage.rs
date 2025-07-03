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

use alloc::borrow::Cow;
use alloc::vec::Vec;

use wasefire_board_api::Singleton;
use wasefire_error::{Code, Error};
use wasefire_store::StorageIndex;

use crate::board::with_state;
use crate::symbol_addr;

pub struct State {
    store: Option<Impl>,
    pub applets: [Raw; 2],
    pub other: Raw,
}

pub fn init() -> State {
    let appleta = symbol_addr!(_appleta) as usize;
    let storea = symbol_addr!(_storea) as usize;
    let limita = symbol_addr!(_limita) as usize;
    let appletb = symbol_addr!(_appletb) as usize;
    let storeb = symbol_addr!(_storeb) as usize;
    let limitb = symbol_addr!(_limitb) as usize;
    let mut pages = Vec::new();
    pages.extend((storea .. limita).step_by(crate::flash::PAGE_SIZE));
    pages.extend((storeb .. limitb).step_by(crate::flash::PAGE_SIZE));
    let store = Some(Impl { pages });
    let applets = [Raw { start: appleta, limit: storea }, Raw { start: appletb, limit: storeb }];
    let start = crate::manifest::inactive() as *const _ as usize;
    let limit = applets[crate::flash::bank_side(start) as usize].start;
    let other = Raw { start, limit };
    State { store, applets, other }
}

pub struct Impl {
    pages: Vec<usize>,
}

impl Singleton for Impl {
    fn take() -> Option<Self> {
        with_state(|state| state.storage.store.take())
    }
}

impl wasefire_store::Storage for Impl {
    fn word_size(&self) -> usize {
        // This is technically wrong. The flash word size is 8. But since it seems we can write up
        // to 4 times per flash word, it's equivalent. And we hide this discrepancy.
        4
    }

    fn page_size(&self) -> usize {
        crate::flash::PAGE_SIZE
    }

    fn num_pages(&self) -> usize {
        self.pages.len()
    }

    fn max_word_writes(&self) -> usize {
        2
    }

    fn max_page_erases(&self) -> usize {
        // The flash theoretically supports 100k erase cycles in high-endurance (instead of 10k),
        // but wasefire-store needs to store this on 16 bits.
        65535
    }

    fn read_slice(&self, index: StorageIndex, length: usize) -> Result<Cow<'_, [u8]>, Error> {
        index.range(length, self)?;
        let ptr = (self.pages[index.page] + index.byte) as *const u8;
        Ok(Cow::Borrowed(unsafe { core::slice::from_raw_parts(ptr, length) }))
    }

    fn write_slice(&mut self, index: StorageIndex, value: &[u8]) -> Result<(), Error> {
        index.range(value.len(), self)?;
        if !index.byte.is_multiple_of(4) || !value.len().is_multiple_of(4) {
            return Err(Error::user(Code::InvalidAlign));
        }
        let addr = self.pages[index.page] + index.byte;
        crate::flash::write(addr, value)
    }

    fn erase_page(&mut self, page: usize) -> Result<(), Error> {
        if page < self.num_pages() {
            crate::flash::erase(self.pages[page])
        } else {
            Err(Error::user(Code::OutOfBounds))
        }
    }
}

pub struct Raw {
    start: usize,
    limit: usize,
}

impl Raw {
    pub fn ptr(&self) -> *const u8 {
        self.start as *const u8
    }

    pub fn len(&self) -> usize {
        self.limit - self.start
    }

    /// Safety: the result is invalidated when written.
    pub unsafe fn read(&self) -> &'static [u8] {
        unsafe { core::slice::from_raw_parts(self.ptr(), self.len()) }
    }

    pub fn write(&self, offset: usize, data: &[u8]) -> Result<(), Error> {
        if offset.checked_add(data.len()).is_none_or(|x| self.len() < x) {
            return Err(Error::user(Code::OutOfBounds));
        }
        crate::flash::write(self.start + offset, data)
    }

    pub fn erase(&self) -> Result<(), Error> {
        for addr in (self.start .. self.limit).step_by(crate::flash::PAGE_SIZE) {
            crate::flash::erase(addr)?;
        }
        Ok(())
    }
}

pub struct Linear {
    dry_run: bool,
    offset: usize,   // aligned to BUFFER_SIZE bytes
    buffer: Vec<u8>, // up to BUFFER_SIZE - 1 unwritten bytes
}

impl Linear {
    pub const BUFFER_SIZE: usize = 8;

    pub fn start(dry_run: bool, raw: &Raw) -> Result<Self, Error> {
        if !dry_run {
            raw.erase()?;
        }
        Ok(Linear { dry_run, offset: 0, buffer: Vec::new() })
    }

    pub fn write(&mut self, raw: &Raw, mut chunk: &[u8]) -> Result<(), Error> {
        let Linear { dry_run, offset, buffer } = self;
        if !buffer.is_empty() {
            let length = core::cmp::min(chunk.len(), Self::BUFFER_SIZE - buffer.len());
            buffer.extend_from_slice(&chunk[.. length]);
            chunk = &chunk[length ..];
            if buffer.len() < Self::BUFFER_SIZE {
                return Ok(());
            }
            if !*dry_run {
                raw.write(*offset, buffer)?;
            }
            *offset += buffer.len();
            buffer.clear();
        }
        if offset.checked_add(chunk.len()).is_none_or(|x| raw.len() - Self::BUFFER_SIZE < x) {
            return Err(Error::user(Code::OutOfBounds));
        }
        let length = chunk.len() / Self::BUFFER_SIZE * Self::BUFFER_SIZE;
        if !*dry_run {
            raw.write(*offset, &chunk[.. length])?;
        }
        *offset += length;
        buffer.extend_from_slice(&chunk[length ..]);
        Ok(())
    }

    /// Flushes and returns the total number of bytes written, unless dry run.
    pub fn flush(self, raw: &Raw) -> Result<Option<usize>, Error> {
        let Linear { dry_run, mut offset, mut buffer } = self;
        if !buffer.is_empty() {
            let length = buffer.len();
            buffer.resize(Self::BUFFER_SIZE, 0xff);
            if !dry_run {
                raw.write(offset, &buffer)?;
            }
            offset += length;
        }
        Ok((!dry_run).then_some(offset))
    }
}
