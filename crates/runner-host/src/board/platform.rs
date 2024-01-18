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

use data_encoding::HEXLOWER_PERMISSIVE;
use wasefire_board_api::platform::Api;
use wasefire_board_api::Error;
use wasefire_error::Code;

pub enum Impl {}

impl Api for Impl {
    fn version(output: &mut [u8]) -> usize {
        const VERSION: Option<&str> = option_env!("WASEFIRE_HOST_VERSION");
        let version = VERSION.unwrap_or_default().as_bytes();
        let version = HEXLOWER_PERMISSIVE.decode(version).expect("--version must be hexadecimal");
        let len = std::cmp::min(output.len(), version.len());
        output[.. len].copy_from_slice(&version[.. len]);
        version.len()
    }

    fn reboot() -> Result<!, Error> {
        Err(Error::world(Code::NotImplemented))
    }
}
