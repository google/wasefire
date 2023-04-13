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

use wasefire_applet_api::button::{self as api, Api};
use wasefire_board_api::button::Api as _;
use wasefire_board_api::Api as Board;

use crate::event::button::Key;
use crate::event::Handler;
use crate::{DispatchSchedulerCall, SchedulerCall, Trap};

pub fn process<B: Board>(call: Api<DispatchSchedulerCall<B>>) {
    match call {
        Api::Count(call) => count(call),
        Api::Register(call) => register(call),
        Api::Unregister(call) => unregister(call),
    }
}

fn count<B: Board>(mut call: SchedulerCall<B, api::count::Sig>) {
    let api::count::Params {} = call.read();
    let count = call.scheduler().board.button().count() as u32;
    call.reply(Ok(api::count::Results { cnt: count.into() }));
}

fn register<B: Board>(mut call: SchedulerCall<B, api::register::Sig>) {
    let api::register::Params { button, handler_func, handler_data } = call.read();
    let button = *button as usize;
    let inst = call.inst();
    let results = try {
        call.scheduler().applet.enable(Handler {
            key: Key { button }.into(),
            inst,
            func: *handler_func,
            data: *handler_data,
        })?;
        call.scheduler().board.button().enable(button).map_err(|_| Trap)?;
        api::register::Results {}
    };
    call.reply(results);
}

fn unregister<B: Board>(mut call: SchedulerCall<B, api::unregister::Sig>) {
    let api::unregister::Params { button } = call.read();
    let button = *button as usize;
    let results = try {
        call.scheduler().board.button().disable(button).map_err(|_| Trap)?;
        call.scheduler().disable_event(Key { button }.into())?;
        api::unregister::Results {}
    };
    call.reply(results);
}
