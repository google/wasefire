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
use core::ptr::addr_of_mut;

use embedded_storage::nor_flash::{
    ErrorType, NorFlash, NorFlashError, NorFlashErrorKind, ReadNorFlash,
};
use nrf52840_hal::nvmc::Nvmc;
use nrf52840_hal::pac::NVMC;
use wasefire_store::{self as store, StorageError, StorageIndex, StorageResult};
use wasefire_sync::{AtomicBool, Ordering, TakeCell};

const PAGE_SIZE: usize = <Nvmc<NVMC>>::ERASE_SIZE;

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
        extern "C" {
            static mut $start: u32;
            static mut $end: u32;
        }
        let start = addr_of_mut!($start) as *mut u8;
        let end = addr_of_mut!($end) as usize;
        let length = end.checked_sub(start as usize).unwrap();
        assert_eq!(length % PAGE_SIZE, 0);
        core::ptr::from_raw_parts_mut(start, length)
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

    /// Returns an exclusive reference to the storage.
    ///
    /// This object is locked until the reference is released with `put()`.
    pub fn take(&self) -> &'static mut [u8] {
        assert!(!self.used.swap(true, Ordering::Acquire));
        unsafe { &mut *self.ptr }
    }

    pub fn put(&self, data: &'static mut [u8]) {
        assert_eq!(data as *mut [u8], self.ptr);
        assert!(self.used.swap(false, Ordering::Release));
    }

    /// Returns a shared reference to the storage.
    ///
    /// # Safety
    ///
    /// The returned reference is invalidated when `take()` is called.
    pub unsafe fn get(&self) -> &'static [u8] {
        assert!(!self.used.load(Ordering::Acquire));
        unsafe { &*self.ptr }
    }

    pub fn len(&self) -> usize {
        self.ptr.len()
    }

    pub const ERASE_SIZE: usize = PAGE_SIZE;
    pub fn erase(&self, offset: usize, length: usize) -> StorageResult<()> {
        let mut helper = Helper::new(self);
        helper.nvmc().erase(offset as u32, (offset + length) as u32).map_err(convert)
    }

    pub const WRITE_SIZE: usize = <Nvmc<NVMC>>::WRITE_SIZE;
    pub fn write(&self, offset: usize, data: &[u8]) -> StorageResult<()> {
        let mut helper = Helper::new(self);
        helper.nvmc().write(offset as u32, data).map_err(convert)
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

impl<'a> Drop for Helper<'a> {
    fn drop(&mut self) {
        let (driver, storage) = self.nvmc.take().unwrap().free();
        DRIVER.put(driver);
        self.storage.put(storage);
    }
}

impl store::Storage for Storage {
    fn word_size(&self) -> usize {
        <Nvmc<NVMC>>::WRITE_SIZE
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

    fn read_slice(&self, index: StorageIndex, length: usize) -> StorageResult<Cow<[u8]>> {
        let offset = offset(self, length, index)?;
        let mut result = vec![0; length];
        let mut helper = Helper::new(self);
        helper.nvmc().read(offset, &mut result).map_err(convert)?;
        Ok(Cow::Owned(result))
    }

    fn write_slice(&mut self, index: StorageIndex, value: &[u8]) -> StorageResult<()> {
        let offset = offset(self, value.len(), index)?;
        let mut helper = Helper::new(self);
        helper.nvmc().write(offset, value).map_err(convert)
    }

    fn erase_page(&mut self, page: usize) -> StorageResult<()> {
        let from = offset(self, PAGE_SIZE, StorageIndex { page, byte: 0 })?;
        let to = from + PAGE_SIZE as u32;
        let mut helper = Helper::new(self);
        helper.nvmc().erase(from, to).map_err(convert)
    }
}

fn offset(storage: &Storage, length: usize, index: StorageIndex) -> StorageResult<u32> {
    Ok(index.range(length, storage)?.start as u32)
}

fn convert(e: <Nvmc<NVMC> as ErrorType>::Error) -> StorageError {
    match e.kind() {
        NorFlashErrorKind::NotAligned => StorageError::NotAligned,
        NorFlashErrorKind::OutOfBounds => StorageError::OutOfBounds,
        _ => StorageError::CustomError,
    }
}
