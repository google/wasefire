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

use header::{Header, Side};
use wasefire_board_api::platform::Api;
use wasefire_board_api::Error;

pub mod update;

pub enum Impl {}

impl Api for Impl {
    type Update = update::Impl;

    fn version(output: &mut [u8]) -> usize {
        let side = Side::current().unwrap();
        let header = Header::new(side);
        const N: usize = 4;
        let len = core::cmp::min(output.len(), N);
        output[.. len].copy_from_slice(&header.timestamp().to_be_bytes()[.. len]);
        N
    }

    fn reboot() -> Result<!, Error> {
        reboot()
    }
}

pub fn reboot() -> ! {
    #[cfg(feature = "debug")]
    defmt::flush();
    nrf52840_hal::pac::SCB::sys_reset()
}
