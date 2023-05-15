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

use alloc::borrow::Cow;

use wasefire_store::{Storage, StorageIndex, StorageResult};

use crate::Unsupported;

impl Storage for Unsupported {
    fn word_size(&self) -> usize {
        unreachable!()
    }

    fn page_size(&self) -> usize {
        unreachable!()
    }

    fn num_pages(&self) -> usize {
        unreachable!()
    }

    fn max_word_writes(&self) -> usize {
        unreachable!()
    }

    fn max_page_erases(&self) -> usize {
        unreachable!()
    }

    fn read_slice(&self, _: StorageIndex, _: usize) -> StorageResult<Cow<[u8]>> {
        unreachable!()
    }

    fn write_slice(&mut self, _: StorageIndex, _: &[u8]) -> StorageResult<()> {
        unreachable!()
    }

    fn erase_page(&mut self, _: usize) -> StorageResult<()> {
        unreachable!()
    }
}
