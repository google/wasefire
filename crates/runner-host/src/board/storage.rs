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

use std::borrow::Cow;

use wasefire_board_api::Singleton;
use wasefire_error::Error;
use wasefire_store::{FileStorage, Storage, StorageIndex};

use crate::with_state;

pub struct Impl(FileStorage);

impl Singleton for Impl {
    fn take() -> Option<Self> {
        with_state(|state| state.storage.take().map(Self))
    }
}

impl Storage for Impl {
    fn word_size(&self) -> usize {
        self.0.word_size()
    }

    fn page_size(&self) -> usize {
        self.0.page_size()
    }

    fn num_pages(&self) -> usize {
        self.0.num_pages()
    }

    fn max_word_writes(&self) -> usize {
        self.0.max_word_writes()
    }

    fn max_page_erases(&self) -> usize {
        self.0.max_page_erases()
    }

    fn read_slice(&self, index: StorageIndex, length: usize) -> Result<Cow<'_, [u8]>, Error> {
        self.0.read_slice(index, length)
    }

    fn write_slice(&mut self, index: StorageIndex, value: &[u8]) -> Result<(), Error> {
        self.0.write_slice(index, value)
    }

    fn erase_page(&mut self, page: usize) -> Result<(), Error> {
        self.0.erase_page(page)
    }
}
