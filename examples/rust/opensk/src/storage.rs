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

use persistent_store::Storage;

pub struct WasefireStorage;

impl Storage for WasefireStorage {
    fn word_size(&self) -> usize {
        todo!()
    }

    fn page_size(&self) -> usize {
        todo!()
    }

    fn num_pages(&self) -> usize {
        todo!()
    }

    fn max_word_writes(&self) -> usize {
        todo!()
    }

    fn max_page_erases(&self) -> usize {
        todo!()
    }

    fn read_slice(
        &self, index: persistent_store::StorageIndex, length: usize,
    ) -> persistent_store::StorageResult<alloc::borrow::Cow<[u8]>> {
        todo!()
    }

    fn write_slice(
        &mut self, index: persistent_store::StorageIndex, value: &[u8],
    ) -> persistent_store::StorageResult<()> {
        todo!()
    }

    fn erase_page(&mut self, page: usize) -> persistent_store::StorageResult<()> {
        todo!()
    }
}

impl Default for WasefireStorage {
    fn default() -> Self {
        Self {}
    }
}
