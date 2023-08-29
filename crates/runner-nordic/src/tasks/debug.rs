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

use wasefire_board_api as board;

pub enum Impl {}

impl board::debug::Api for Impl {
    #[cfg(feature = "debug")]
    const MAX_TIME: u64 = 0x00ffffff_ffffffff;
    #[cfg(feature = "release")]
    const MAX_TIME: u64 = 0;

    fn time() -> u64 {
        #[cfg(feature = "debug")]
        let time = crate::systick::uptime_us();
        #[cfg(feature = "release")]
        let time = 0;
        time
    }

    fn exit(success: bool) -> ! {
        if success {
            loop {
                cortex_m::asm::bkpt();
            }
        } else {
            #[cfg(feature = "debug")]
            panic_probe::hard_fault();
            #[cfg(feature = "release")]
            panic!();
        }
    }
}
