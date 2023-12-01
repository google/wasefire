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
use core::slice;

use embedded_storage::nor_flash::{
    ErrorType, NorFlash, NorFlashError, NorFlashErrorKind, ReadNorFlash,
};
use nrf52840_hal::nvmc::Nvmc;
use nrf52840_hal::pac::NVMC;
use wasefire_store::{self as store, StorageError, StorageIndex, StorageResult};
use wasefire_sync::TakeCell;

const PAGE_SIZE: usize = <Nvmc<NVMC>>::ERASE_SIZE;

static DRIVER: TakeCell<NVMC> = TakeCell::new(None);

pub fn init(nvmc: NVMC) {
    DRIVER.put(nvmc);
}

pub struct Storage(TakeCell<&'static mut [u8]>);

macro_rules! take_storage {
    ($start:ident .. $end:ident) => {{
        assert!(!wasefire_sync::executed!());
        extern "C" {
            static mut $start: u32;
            static mut $end: u32;
        }
        let start = unsafe { &mut $start as *mut u32 as *mut u8 };
        let end = unsafe { &mut $end as *mut u32 as usize };
        let length = end.checked_sub(start as usize).unwrap();
        assert_eq!(length % PAGE_SIZE, 0);
        unsafe { slice::from_raw_parts_mut(start, length) }
    }};
}

impl Storage {
    pub fn new_store() -> Self {
        Storage(TakeCell::new(Some(take_storage!(__sstore .. __estore))))
    }

    pub fn new_other() -> Self {
        Storage(TakeCell::new(Some(take_storage!(__sother .. __eother))))
    }

    pub fn as_ptr(&self) -> *const u8 {
        self.0.with(|x| x.as_ptr())
    }

    pub fn len(&self) -> usize {
        self.0.with(|x| x.len())
    }
}

struct Helper<'a> {
    storage: &'a TakeCell<&'static mut [u8]>,
    nvmc: Option<Nvmc<NVMC>>,
}

impl<'a> Helper<'a> {
    fn new(storage: &'a TakeCell<&'static mut [u8]>) -> Self {
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
        let mut helper = Helper::new(&self.0);
        helper.nvmc().read(offset, &mut result).map_err(convert)?;
        Ok(Cow::Owned(result))
    }

    fn write_slice(&mut self, index: StorageIndex, value: &[u8]) -> StorageResult<()> {
        let offset = offset(self, value.len(), index)?;
        let mut helper = Helper::new(&self.0);
        helper.nvmc().write(offset, value).map_err(convert)
    }

    fn erase_page(&mut self, page: usize) -> StorageResult<()> {
        let from = offset(self, PAGE_SIZE, StorageIndex { page, byte: 0 })?;
        let to = from + PAGE_SIZE as u32;
        let mut helper = Helper::new(&self.0);
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
