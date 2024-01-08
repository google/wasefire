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

use wasefire_applet_api::led::{self as api, Api};
#[cfg(feature = "board-api-led")]
use wasefire_board_api::led::Api as _;
use wasefire_board_api::Api as Board;
#[cfg(feature = "board-api-led")]
use wasefire_board_api::{self as board, Id, Support};

#[cfg(feature = "board-api-led")]
use crate::Trap;
use crate::{DispatchSchedulerCall, SchedulerCall};

pub fn process<B: Board>(call: Api<DispatchSchedulerCall<B>>) {
    match call {
        Api::Count(call) => count(call),
        Api::Get(call) => or_trap!("board-api-led", get(call)),
        Api::Set(call) => or_trap!("board-api-led", set(call)),
    }
}

fn count<B: Board>(call: SchedulerCall<B, api::count::Sig>) {
    let api::count::Params {} = call.read();
    #[cfg(feature = "board-api-led")]
    let count = board::Led::<B>::SUPPORT as u32;
    #[cfg(not(feature = "board-api-led"))]
    let count = 0;
    call.reply(Ok(api::count::Results { cnt: count.into() }));
}

#[cfg(feature = "board-api-led")]
fn get<B: Board>(call: SchedulerCall<B, api::get::Sig>) {
    let api::get::Params { led } = call.read();
    let results = try {
        let id = Id::new(*led as usize).ok_or(Trap)?;
        let status = match board::Led::<B>::get(id).map_err(|_| Trap)? {
            false => api::Status::Off.into(),
            true => api::Status::On.into(),
        };
        api::get::Results { status }
    };
    call.reply(results);
}

#[cfg(feature = "board-api-led")]
fn set<B: Board>(call: SchedulerCall<B, api::set::Sig>) {
    let api::set::Params { led, status } = call.read();
    let results = try {
        let id = Id::new(*led as usize).ok_or(Trap)?;
        let on = matches!(api::Status::try_from(*status)?, api::Status::On);
        board::Led::<B>::set(id, on).map_err(|_| Trap)?;
        api::set::Results {}
    };
    call.reply(results);
}
