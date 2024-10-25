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

use wasefire_wire::Wire;

/// Requests to transfer data.
///
/// The responses contain no information and just use the unit type.
#[derive(Debug, Wire)]
pub enum Request<'a> {
    /// Starts a transfer.
    Start {
        /// Whether the transfer is a dry-run.
        ///
        /// In dry-run mode, all mutable operations are skipped.
        dry_run: bool,
    },

    /// Writes a chunk of data.
    ///
    /// Should only be used between a `Start` and a `Finish`.
    Write {
        /// The next chunk of data to transfer.
        chunk: &'a [u8],
    },

    /// Finishes a transfer.
    Finish,
}
