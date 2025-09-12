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
use wasefire_common::addr_of_symbol;
use wasefire_error::{Code, Error};
use wasefire_store::StorageIndex;

use crate::board::with_state;
use crate::flash::PAGE_SIZE;

pub struct State {
    store: Option<Impl>,
    pub applets: [Raw; 2],
    pub other: Raw,
}

pub fn init() -> State {
    let appleta = addr_of_symbol!(_appleta);
    let storea = addr_of_symbol!(_storea);
    let limita = addr_of_symbol!(_limita);
    let appletb = addr_of_symbol!(_appletb);
    let storeb = addr_of_symbol!(_storeb);
    let limitb = addr_of_symbol!(_limitb);
    let mut pages = Vec::new();
    pages.extend((storea .. limita).step_by(PAGE_SIZE));
    pages.extend((storeb .. limitb).step_by(PAGE_SIZE));
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
        PAGE_SIZE
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

    pub fn erase(&self, offset: usize) -> Result<(), Error> {
        crate::flash::erase(self.start + offset)
    }
}

pub struct Linear {
    dry_run: bool,
    length: Option<usize>,
    offset: usize,
}

impl Linear {
    pub fn start(dry_run: bool) -> Result<Self, Error> {
        Ok(Linear { dry_run, length: None, offset: 0 })
    }

    pub fn erase(&mut self, raw: &Raw) -> Result<(), Error> {
        if self.length.is_some() {
            return Err(Error::user(Code::InvalidState));
        }
        if !self.dry_run
            && !unsafe { raw.read() }[self.offset ..][.. PAGE_SIZE].iter().all(|x| *x == 0xff)
        {
            raw.erase(self.offset)?;
        }
        self.offset += PAGE_SIZE;
        if raw.len() <= self.offset {
            Error::internal(Code::InvalidLength).check(self.offset == raw.len())?;
            self.length = Some(0);
            self.offset = 0;
        }
        Ok(())
    }

    pub fn write(&mut self, raw: &Raw, chunk: &[u8]) -> Result<(), Error> {
        Error::user(Code::InvalidLength).check(chunk.len() <= PAGE_SIZE)?;
        Error::user(Code::OutOfBounds).check(self.offset + PAGE_SIZE <= raw.len())?;
        let length = self.length.as_mut().ok_or(Error::user(Code::InvalidState))?;
        if !self.dry_run {
            let len = chunk.len() / 8 * 8;
            let (aligned, rest) = chunk.split_at(len);
            raw.write(self.offset, aligned)?;
            if !rest.is_empty() {
                let mut word = [0xff; 8];
                word[.. rest.len()].copy_from_slice(rest);
                raw.write(self.offset + len, &word)?;
            }
        }
        self.offset += PAGE_SIZE;
        *length += chunk.len();
        Ok(())
    }

    /// Flushes and returns the total number of bytes written, unless dry run.
    pub fn finish(self) -> Result<Option<usize>, Error> {
        let length = self.length.ok_or(Error::user(Code::InvalidState))?;
        Ok((!self.dry_run).then_some(length))
    }
}
