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

//! Common transfer interface.

use crate::Error;

/// Common transfer interface.
///
/// A transfer process follows these steps:
/// - `start` is called once
/// - `erase` is called `start()` times
/// - `write` is called zero or more times
/// - `finish` is called once
pub trait Api: Send {
    /// Complete chunk size in bytes.
    const CHUNK_SIZE: usize;

    /// Starts a transfer process and returns the number of pages to erase.
    ///
    /// The destination is not modified if `dry-run` is set.
    fn start(dry_run: bool) -> Result<usize, Error>;

    /// Erases a page of the destination.
    fn erase() -> Result<(), Error>;

    /// Writes a chunk to the destination.
    ///
    /// All chunks but the last must be complete.
    fn write(chunk: &[u8]) -> Result<(), Error>;

    /// Finishes a transfer process.
    fn finish() -> Result<(), Error>;
}
