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

use wasefire_applet_api::debug::{self as api, Api, Perf};
use wasefire_board_api::debug::Api as _;
use wasefire_board_api::{self as board, Api as Board};
use wasefire_logger as logger;

use crate::{DispatchSchedulerCall, SchedulerCall, Trap};

pub fn process<B: Board>(call: Api<DispatchSchedulerCall<B>>) {
    match call {
        Api::Println(call) => println(call),
        Api::Time(call) => time(call),
        Api::Perf(call) => perf(call),
        Api::Exit(call) => exit(call),
    }
}

fn println<B: Board>(mut call: SchedulerCall<B, api::println::Sig>) {
    let api::println::Params { ptr, len } = call.read();
    let memory = call.memory();
    let results = try {
        let time = board::Debug::<B>::time();
        let message = core::str::from_utf8(memory.get(*ptr, *len)?).map_err(|_| Trap)?;
        logger::println!("{}.{:06}: {}", time / 1000000, time % 1000000, message);
        api::println::Results {}
    };
    call.reply(results)
}

fn time<B: Board>(mut call: SchedulerCall<B, api::time::Sig>) {
    let api::time::Params { ptr } = call.read();
    let memory = call.memory();
    let results = try {
        let time = board::Debug::<B>::time();
        let high = (time >> 32) as u32;
        let low = time as u32;
        if *ptr != 0 {
            memory.get_mut(*ptr, 4)?.copy_from_slice(&high.to_le_bytes());
        }
        api::time::Results { res: low.into() }
    };
    call.reply(results)
}

fn perf<B: Board>(mut call: SchedulerCall<B, api::perf::Sig>) {
    let api::perf::Params { ptr } = call.read();
    #[cfg(feature = "debug")]
    let perf = call.scheduler().perf.read();
    #[cfg(not(feature = "debug"))]
    let perf = Perf { platform: 0, applets: 0, waiting: 0 };
    let memory = call.memory();
    let results = try {
        *memory.from_bytes_mut::<Perf>(*ptr)? = perf;
        api::perf::Results {}
    };
    call.reply(results)
}

fn exit<B: Board>(call: SchedulerCall<B, api::exit::Sig>) {
    let api::exit::Params { code } = call.read();
    board::Debug::<B>::exit(*code == 0);
}
