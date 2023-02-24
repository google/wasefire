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
use core::cell::RefCell;
use core::slice;

use embedded_storage::nor_flash::{
    ErrorType, NorFlash, NorFlashError, NorFlashErrorKind, ReadNorFlash,
};
use nrf52840_hal::nvmc::Nvmc;
use nrf52840_hal::pac::NVMC;
use wasefire_store::{self as store, StorageError, StorageIndex, StorageResult};

const PAGE_SIZE: usize = <Nvmc<NVMC>>::ERASE_SIZE;

pub struct Storage(RefCell<Nvmc<NVMC>>);

impl Storage {
    pub fn new(nvmc: NVMC) -> Self {
        // SAFETY: We assume only one NVMC instance can exist, so this function is called at most
        // once, and so we call inner at most once.
        Storage(RefCell::new(Nvmc::new(nvmc, unsafe { Self::inner() })))
    }

    // SAFETY: Must be called at most once.
    unsafe fn inner() -> &'static mut [u8] {
        extern "C" {
            static mut __sstore: u32;
            static mut __estore: u32;
        }
        let start = &mut __sstore as *mut u32 as *mut u8;
        let sstore = start as usize;
        let estore = &mut __estore as *mut u32 as usize;
        assert!(sstore < estore);
        let length = estore - sstore;
        assert_eq!(length % PAGE_SIZE, 0);
        slice::from_raw_parts_mut(start, length)
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
        self.0.borrow().capacity() / PAGE_SIZE
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
        self.0.borrow_mut().read(offset, &mut result).map_err(convert)?;
        Ok(Cow::Owned(result))
    }

    fn write_slice(&mut self, index: StorageIndex, value: &[u8]) -> StorageResult<()> {
        let offset = offset(self, value.len(), index)?;
        self.0.get_mut().write(offset, value).map_err(convert)
    }

    fn erase_page(&mut self, page: usize) -> StorageResult<()> {
        let from = offset(self, PAGE_SIZE, StorageIndex { page, byte: 0 })?;
        let to = from + PAGE_SIZE as u32;
        self.0.get_mut().erase(from, to).map_err(convert)
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
