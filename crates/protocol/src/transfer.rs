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

//! Protocol to transfer data from the host to the device.

use alloc::borrow::Cow;

use wasefire_wire::Wire;

/// Requests to transfer data.
#[derive(Debug, Wire)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Request<'a> {
    /// Starts a transfer.
    Start {
        /// Whether the transfer is a dry-run.
        ///
        /// In dry-run mode, nothing is erased or written, but the process is otherwise unchanged.
        dry_run: bool,
    },

    /// Erases a page.
    ///
    /// See [`Response::Start::chunk_size`] for the number of pages to erase.
    Erase,

    /// Writes a chunk of data.
    Write {
        /// The next chunk of data to transfer.
        ///
        /// It should have [`Response::Start::chunk_size`] bytes unless it's the last chunk.
        chunk: Cow<'a, [u8]>,
    },

    /// Finishes a transfer.
    Finish,
}

#[derive(Debug, Wire)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Response {
    Start {
        /// Size in bytes of a complete chunk.
        chunk_size: usize,

        /// Number of pages to erase.
        num_pages: usize,
    },
    Erase,
    Write,
    Finish,
}

#[derive(Debug, Wire)]
#[cfg(feature = "host")]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum _Request0<'a> {
    Start { dry_run: bool },
    Write { chunk: Cow<'a, [u8]> },
    Finish,
}
