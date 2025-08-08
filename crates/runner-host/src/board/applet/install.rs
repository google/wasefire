// Copyright 2025 Google LLC
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

use anyhow::Result;
use tokio::runtime::Handle;
use wasefire_board_api::transfer::Api;
use wasefire_error::{Code, Error};

use super::with_state;

pub enum Impl {}

impl Api for Impl {
    const CHUNK_SIZE: usize = 4096;

    fn start(dry_run: bool) -> Result<usize, Error> {
        with_state(|x| Handle::current().block_on(x.start(dry_run)))?;
        Ok(0)
    }

    fn erase() -> Result<(), Error> {
        Err(Error::user(Code::InvalidState))
    }

    fn write(chunk: &[u8]) -> Result<(), Error> {
        with_state(|x| x.write(chunk))
    }

    fn finish() -> Result<(), Error> {
        with_state(|x| Handle::current().block_on(x.finish()))
    }
}
