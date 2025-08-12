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

use wasefire_applet_api::clock::Api;
#[cfg(feature = "board-api-clock")]
use wasefire_applet_api::clock::{self as api};
use wasefire_board_api::Api as Board;
#[cfg(feature = "board-api-clock")]
use wasefire_board_api::applet::Memory as _;
#[cfg(feature = "board-api-clock")]
use wasefire_board_api::{self as board, clock::Api as _};

use crate::DispatchSchedulerCall;
#[cfg(feature = "board-api-clock")]
use crate::SchedulerCall;

pub fn process<B: Board>(call: Api<DispatchSchedulerCall<B>>) {
    match call {
        Api::UptimeUs(call) => or_fail!("board-api-clock", uptime_us(call)),
    }
}

#[cfg(feature = "board-api-clock")]
fn uptime_us<B: Board>(mut call: SchedulerCall<B, api::uptime_us::Sig>) {
    let api::uptime_us::Params { ptr } = call.read();
    let memory = call.memory();
    let result = try {
        let time = board::Clock::<B>::uptime_us()?;
        memory.get_mut(*ptr, 8)?.copy_from_slice(&time.to_le_bytes());
    };
    call.reply(result);
}
