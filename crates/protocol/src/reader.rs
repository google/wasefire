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
    pub fn new(data: &'a [u8]) -> Self {
        Reader(data)
    }

    pub fn finalize(self) -> Result<(), Error> {
        Error::user(Code::InvalidLength).check(self.0.is_empty())
    }

    fn split_at(&mut self, offset: usize) -> Result<&'a [u8], Error> {
        Error::user(Code::InvalidLength).check(offset <= self.0.len())?;
        let (head, tail) = self.0.split_at(offset);
        self.0 = tail;
        Ok(head)
    }

    pub fn get_all(&mut self) -> &'a [u8] {
        self.split_at(self.0.len()).unwrap()
    }

    pub fn get_u8(&mut self) -> Result<u8, Error> {
        Ok(self.split_at(1)?[0])
    }

    pub fn get_u16(&mut self) -> Result<u16, Error> {
        Ok(u16::from_be_bytes(self.split_at(2)?.try_into().unwrap()))
    }
}
