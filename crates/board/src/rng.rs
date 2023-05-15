// Copyright 2022 Google LLC
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

//! Random number generator interface.

use crate::{Error, Unsupported};

/// Random number generator interface.
pub trait Api {
    /// Fills a buffer with random bytes uniformly.
    fn fill_bytes(buffer: &mut [u8]) -> Result<(), Error>;
}

impl Api for Unsupported {
    fn fill_bytes(_: &mut [u8]) -> Result<(), Error> {
        Err(Error::World)
    }
}
