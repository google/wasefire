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

use wasefire_board_api::transfer::Api;
use wasefire_error::Error;

use super::{STATE, read_size, write_size};

pub enum Impl {}

impl Api for Impl {
    const CHUNK_SIZE: usize = crate::storage::PAGE_SIZE;

    fn start(dry_run: bool) -> Result<usize, Error> {
        STATE.with(|state| {
            if !dry_run {
                state.writer.erase_last_page()?;
            }
            state.size = 0;
            state.writer.start(dry_run)
        })
    }

    fn erase() -> Result<(), Error> {
        STATE.with(|state| state.writer.erase())
    }

    fn write(chunk: &[u8]) -> Result<(), Error> {
        STATE.with(|state| {
            state.size += chunk.len();
            state.writer.write(chunk)
        })
    }

    fn finish() -> Result<(), Error> {
        STATE.with(|state| {
            let dry_run = state.writer.dry_run()?;
            state.writer.finish()?;
            if dry_run {
                state.size = read_size(state.writer.storage())?;
                Ok(())
            } else {
                write_size(state.writer.storage_mut(), state.size)
            }
        })
    }
}
