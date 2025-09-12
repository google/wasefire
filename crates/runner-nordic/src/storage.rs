// Copyright 2022 Google LLC
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
use alloc::vec;

use embedded_storage::nor_flash::{
    ErrorType, NorFlash, NorFlashError, NorFlashErrorKind, ReadNorFlash,
};
use nrf52840_hal::nvmc::Nvmc;
use nrf52840_hal::pac::NVMC;
use wasefire_common::addr_of_symbol;
use wasefire_error::{Code, Error};
use wasefire_store::{self as store, StorageIndex};
use wasefire_sync::{AtomicBool, Ordering, TakeCell};

const WORD_SIZE: usize = <Nvmc<NVMC>>::WRITE_SIZE;
pub const PAGE_SIZE: usize = <Nvmc<NVMC>>::ERASE_SIZE;

static DRIVER: TakeCell<NVMC> = TakeCell::new(None);

pub fn init(nvmc: NVMC) {
    DRIVER.put(nvmc);
}

pub struct Storage {
    ptr: *mut [u8],
    used: AtomicBool,
}

unsafe impl Send for Storage {}

macro_rules! take_storage {
    ($start:ident .. $end:ident) => {{
        assert!(!wasefire_sync::executed!());
        let start = addr_of_symbol!($start);
        let end = addr_of_symbol!($end);
        assert_eq!(start % PAGE_SIZE, 0);
        assert_eq!(end % PAGE_SIZE, 0);
        let length = end.checked_sub(start).unwrap();
        core::ptr::from_raw_parts_mut(start as *mut u8, length)
    }};
}

impl Storage {
    fn new(ptr: *mut [u8]) -> Self {
        Storage { ptr, used: AtomicBool::new(false) }
    }

    pub fn new_store() -> Self {
        Storage::new(take_storage!(__sstore .. __estore))
    }

    pub fn new_other() -> Self {
        Storage::new(take_storage!(__sother .. __eother))
    }

    pub fn new_applet() -> Self {
        Storage::new(take_storage!(__sapplet .. __eapplet))
    }

    /// Returns an exclusive reference to the storage.
    ///
    /// This object is locked until the reference is released with `put()`.
    fn take(&self) -> &'static mut [u8] {
        assert!(!self.used.swap(true, Ordering::Acquire));
        unsafe { &mut *self.ptr }
    }

    fn put(&self, data: &'static mut [u8]) {
        assert_eq!(core::ptr::from_mut(data), self.ptr);
        assert!(self.used.swap(false, Ordering::Release));
    }

    /// Returns a shared reference to the storage.
    ///
    /// # Safety
    ///
    /// The returned reference is invalidated when `take()` is called.
    unsafe fn get(&self) -> &'static [u8] {
        assert!(!self.used.load(Ordering::Acquire));
        unsafe { &*self.ptr }
    }

    fn len(&self) -> usize {
        self.ptr.len()
    }
}

pub struct StorageWriter {
    storage: Storage,
    state: Option<WriterState>,
}

struct WriterState {
    dry_run: bool,
    write: bool,
    offset: usize,
}

impl StorageWriter {
    pub fn new(storage: Storage) -> Self {
        StorageWriter { storage, state: None }
    }

    pub fn storage(&self) -> &Storage {
        &self.storage
    }

    pub fn storage_mut(&mut self) -> &mut Storage {
        &mut self.storage
    }

    /// Returns a shared reference to the storage.
    ///
    /// # Safety
    ///
    /// The returned reference is invalidated when `start()` is called.
    pub unsafe fn get(&self) -> Result<&'static [u8], Error> {
        if self.state.is_some() {
            return Err(Error::user(Code::InvalidState));
        }
        Ok(unsafe { self.storage.get() })
    }

    pub fn erase_last_page(&mut self) -> Result<(), Error> {
        let to = self.storage.len() as u32;
        let from = to - PAGE_SIZE as u32;
        Helper::new(&self.storage).nvmc().erase(from, to).map_err(convert)
    }

    pub fn dry_run(&self) -> Result<bool, Error> {
        self.state.as_ref().map(|x| x.dry_run).ok_or(Error::user(Code::InvalidState))
    }

    pub fn start(&mut self, dry_run: bool) -> Result<usize, Error> {
        self.state = Some(WriterState { dry_run, write: false, offset: 0 });
        Ok(self.storage.len() / PAGE_SIZE)
    }

    pub fn erase(&mut self) -> Result<(), Error> {
        let (storage, state) = self.state()?;
        if state.write {
            return Err(Error::user(Code::InvalidState));
        }
        if !state.dry_run {
            let content = unsafe { storage.get() };
            if !content[state.offset ..][.. PAGE_SIZE].iter().all(|x| *x == 0xff) {
                let from = state.offset as u32;
                let to = from + PAGE_SIZE as u32;
                Helper::new(storage).nvmc().erase(from, to).map_err(convert)?;
            }
        }
        state.offset += PAGE_SIZE;
        if storage.len() <= state.offset {
            Error::internal(Code::InvalidLength).check(state.offset == storage.len())?;
            state.write = true;
            state.offset = 0;
        }
        Ok(())
    }

    pub fn write(&mut self, chunk: &[u8]) -> Result<(), Error> {
        let (storage, state) = self.state()?;
        Error::user(Code::InvalidLength).check(chunk.len() <= PAGE_SIZE)?;
        Error::user(Code::OutOfBounds).check(state.offset + PAGE_SIZE <= storage.len())?;
        Error::user(Code::InvalidState).check(state.write)?;
        if !state.dry_run {
            let mut pos = state.offset as u32;
            let len = chunk.len() / 4 * 4;
            let (aligned, rest) = chunk.split_at(len);
            Helper::new(storage).nvmc().write(pos, aligned).map_err(convert)?;
            if !rest.is_empty() {
                pos += len as u32;
                let mut word = [0xff; 4];
                word[.. rest.len()].copy_from_slice(rest);
                Helper::new(storage).nvmc().write(pos, &word).map_err(convert)?;
            }
        }
        state.offset += PAGE_SIZE;
        Ok(())
    }

    pub fn finish(&mut self) -> Result<(), Error> {
        let state = self.state.take().ok_or(Error::user(Code::InvalidState))?;
        Error::user(Code::InvalidState).check(state.write)
    }

    fn state(&mut self) -> Result<(&mut Storage, &mut WriterState), Error> {
        Ok((&mut self.storage, self.state.as_mut().ok_or(Error::user(Code::InvalidState))?))
    }
}

struct Helper<'a> {
    storage: &'a Storage,
    nvmc: Option<Nvmc<NVMC>>,
}

impl<'a> Helper<'a> {
    fn new(storage: &'a Storage) -> Self {
        let nvmc = Some(Nvmc::new(DRIVER.take(), storage.take()));
        Helper { storage, nvmc }
    }

    fn nvmc(&mut self) -> &mut Nvmc<NVMC> {
        self.nvmc.as_mut().unwrap()
    }
}

impl Drop for Helper<'_> {
    fn drop(&mut self) {
        let (driver, storage) = self.nvmc.take().unwrap().free();
        DRIVER.put(driver);
        self.storage.put(storage);
    }
}

impl store::Storage for Storage {
    fn word_size(&self) -> usize {
        WORD_SIZE
    }

    fn page_size(&self) -> usize {
        PAGE_SIZE
    }

    fn num_pages(&self) -> usize {
        self.len() / PAGE_SIZE
    }

    fn max_word_writes(&self) -> usize {
        2
    }

    fn max_page_erases(&self) -> usize {
        10000
    }

    fn read_slice(&self, index: StorageIndex, length: usize) -> Result<Cow<'_, [u8]>, Error> {
        let offset = offset(self, length, index)?;
        let mut result = vec![0; length];
        let mut helper = Helper::new(self);
        helper.nvmc().read(offset, &mut result).map_err(convert)?;
        Ok(Cow::Owned(result))
    }

    fn write_slice(&mut self, index: StorageIndex, value: &[u8]) -> Result<(), Error> {
        let offset = offset(self, value.len(), index)?;
        let mut helper = Helper::new(self);
        helper.nvmc().write(offset, value).map_err(convert)
    }

    fn erase_page(&mut self, page: usize) -> Result<(), Error> {
        let from = offset(self, PAGE_SIZE, StorageIndex { page, byte: 0 })?;
        let to = from + PAGE_SIZE as u32;
        let mut helper = Helper::new(self);
        helper.nvmc().erase(from, to).map_err(convert)
    }
}

fn offset(storage: &Storage, length: usize, index: StorageIndex) -> Result<u32, Error> {
    Ok(index.range(length, storage)?.start as u32)
}

fn convert(e: <Nvmc<NVMC> as ErrorType>::Error) -> Error {
    match e.kind() {
        NorFlashErrorKind::NotAligned => Error::user(Code::InvalidAlign),
        NorFlashErrorKind::OutOfBounds => Error::user(Code::OutOfBounds),
        _ => Error::world(0),
    }
}
