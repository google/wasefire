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
use wasefire_board_api::led::Api as Bpi;
use wasefire_board_api::Api as Board;

use crate::{DispatchSchedulerCall, SchedulerCall, Trap};

pub fn process<B: Board>(call: Api<DispatchSchedulerCall<B>>) {
    match call {
        Api::Count(call) => count(call),
        Api::Get(call) => get(call),
        Api::Set(call) => set(call),
    }
}

fn count<B: Board>(mut call: SchedulerCall<B, api::count::Sig>) {
    let api::count::Params {} = call.read();
    let count = <B as Bpi>::count(&mut call.scheduler().board) as u32;
    call.reply(Ok(api::count::Results { cnt: count.into() }));
}

fn get<B: Board>(mut call: SchedulerCall<B, api::get::Sig>) {
    let api::get::Params { led } = call.read();
    let results = try {
        let status =
            match <B as Bpi>::get(&mut call.scheduler().board, *led as usize).map_err(|_| Trap)? {
                false => api::Status::Off.into(),
                true => api::Status::On.into(),
            };
        api::get::Results { status }
    };
    call.reply(results);
}

fn set<B: Board>(mut call: SchedulerCall<B, api::set::Sig>) {
    let api::set::Params { led, status } = call.read();
    let results = try {
        let on = matches!(api::Status::try_from(*status)?, api::Status::On);
        <B as Bpi>::set(&mut call.scheduler().board, *led as usize, on).map_err(|_| Trap)?;
        api::set::Results {}
    };
    call.reply(results);
}
