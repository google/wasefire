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

use alloc::vec::Vec;

#[derive(Default)]
pub struct Writer<'a> {
    owned: Vec<u8>,
    chunks: Vec<Chunk<'a>>,
}

impl<'a> Writer<'a> {
    pub(crate) fn new() -> Self {
        Writer::default()
    }

    pub(crate) fn put_share(&mut self, data: &'a [u8]) {
        self.chunks.push(Chunk::Borrowed(data));
    }

    pub(crate) fn put_copy(&mut self, data: &[u8]) {
        let offset = self.owned.len();
        self.owned.extend_from_slice(data);
        let length = data.len();
        self.chunks.push(Chunk::Owned { offset, length });
    }

    pub(crate) fn finalize(self, data: &mut Vec<u8>) {
        data.reserve_exact(self.chunks.iter().map(|x| x.len()).sum());
        for chunk in self.chunks {
            data.extend_from_slice(chunk.slice(&self.owned));
        }
    }
}

enum Chunk<'a> {
    Owned { offset: usize, length: usize },
    Borrowed(&'a [u8]),
}

impl<'a> Chunk<'a> {
    fn slice(&self, owned: &'a [u8]) -> &'a [u8] {
        match *self {
            Chunk::Owned { offset, length } => &owned[offset ..][.. length],
            Chunk::Borrowed(x) => x,
        }
    }

    fn len(&self) -> usize {
        match *self {
            Chunk::Owned { length, .. } => length,
            Chunk::Borrowed(x) => x.len(),
        }
    }
}
