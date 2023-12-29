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

//! Platform update interface.

use alloc::boxed::Box;

use wasefire_error::Code;

use crate::{Error, Support, Unsupported};

/// Platform update interface.
pub trait Api: Support<bool> + Send {
    /// Returns the metadata of the platform.
    ///
    /// This typically contains the version and side (A or B) of the running platform.
    fn metadata() -> Result<Box<[u8]>, Error>;

    /// Starts a platform update process.
    ///
    /// During a dry-run, any mutable operation is skipped and only checks are performed.
    fn initialize(dry_run: bool) -> Result<(), Error>;

    /// Processes the next chunk of a platform update.
    fn process(chunk: &[u8]) -> Result<(), Error>;

    /// Finalizes a platform update process.
    ///
    /// This function will reboot when the update is successful and thus only returns in case of
    /// errors or in dry-run mode.
    fn finalize() -> Result<(), Error>;
}

impl Api for Unsupported {
    fn metadata() -> Result<Box<[u8]>, Error> {
        Err(Error::user(Code::NotImplemented))
    }

    fn initialize(_: bool) -> Result<(), Error> {
        Err(Error::user(Code::NotImplemented))
    }

    fn process(_: &[u8]) -> Result<(), Error> {
        Err(Error::user(Code::NotImplemented))
    }

    fn finalize() -> Result<(), Error> {
        Err(Error::user(Code::NotImplemented))
    }
}
