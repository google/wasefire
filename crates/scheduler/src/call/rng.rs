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

use wasefire_applet_api::rng::{self as api, Api};
use wasefire_board_api::rng::Api as Bpi;
use wasefire_board_api::Api as Board;

use crate::{DispatchSchedulerCall, SchedulerCall, Trap};

pub fn process<B: Board>(call: Api<DispatchSchedulerCall<B>>) {
    match call {
        Api::FillBytes(call) => fill_bytes(call),
    }
}

fn fill_bytes<B: Board>(mut call: SchedulerCall<B, api::fill_bytes::Sig>) {
    let api::fill_bytes::Params { ptr, len } = call.read();
    let scheduler = call.scheduler();
    let memory = scheduler.applet.memory();
    let results = try {
        let output = memory.get_mut(*ptr, *len)?;
        <B as Bpi>::fill_bytes(&mut scheduler.board, output).map_err(|_| Trap)?;
        api::fill_bytes::Results {}
    };
    call.reply(results);
}
