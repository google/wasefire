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

use wasefire_error::{Code, Error};

pub struct Reader<'a>(&'a [u8]);

impl<'a> Reader<'a> {
    pub(crate) fn new(data: &'a [u8]) -> Self {
        Reader(data)
    }

    pub(crate) fn get(&mut self, offset: usize) -> Result<&'a [u8], Error> {
        Error::user(Code::InvalidLength).check(offset <= self.0.len())?;
        let head;
        (head, self.0) = self.0.split_at(offset);
        Ok(head)
    }

    pub(crate) fn finalize(self) -> &'a [u8] {
        self.0
    }
}
