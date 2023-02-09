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

use ::api::debug::{self as api, Api};
use board::Api as Board;

use crate::{DispatchSchedulerCall, SchedulerCall, Trap};

pub fn process<B: Board>(call: Api<DispatchSchedulerCall<B>>) {
    match call {
        Api::Println(call) => println(call),
    }
}

fn println<B: Board>(mut call: SchedulerCall<B, api::println::Sig>) {
    let api::println::Params { ptr, len } = call.read();
    let memory = call.memory();
    let results = try {
        logger::println!("{}", core::str::from_utf8(memory.get(*ptr, *len)?).map_err(|_| Trap)?);
        api::println::Results {}
    };
    call.reply(results)
}
