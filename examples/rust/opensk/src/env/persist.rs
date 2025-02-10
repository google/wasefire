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
// limitations under the License.use opensk_lib::api::clock::Clock;

use alloc::vec::Vec;

use opensk_lib::api::persist::{Persist, PersistIter};
use opensk_lib::ctap::status_code::CtapResult;

use crate::env::WasefireEnv;

impl Persist for WasefireEnv {
    fn find(&self, _key: usize) -> CtapResult<Option<Vec<u8>>> {
        todo!()
    }

    fn insert(&mut self, _key: usize, _value: &[u8]) -> CtapResult<()> {
        todo!()
    }

    fn remove(&mut self, _key: usize) -> CtapResult<()> {
        todo!()
    }

    fn iter(&self) -> CtapResult<PersistIter<'_>> {
        todo!()
    }
}
