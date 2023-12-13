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

use wasefire_applet_api::radio::{self as api, Api};
use wasefire_board_api::radio::Api as _;
use wasefire_board_api::{self as board, Api as Board};

use crate::applet::store::MemoryApi;
use crate::event::radio::Key;
use crate::event::Handler;
use crate::{DispatchSchedulerCall, SchedulerCall, Trap};

pub fn process<B: Board>(call: Api<DispatchSchedulerCall<B>>) {
    match call {
        Api::Register(call) => register(call),
        Api::Unregister(call) => unregister(call),
        Api::Read(call) => read(call),
    }
}

fn register<B: Board>(mut call: SchedulerCall<B, api::register::Sig>) {
    let api::register::Params { handler_func, handler_data } = call.read();
    let inst = call.inst();
    let results = try {
        call.scheduler().applet.enable(Handler {
            key: Key::Received.into(),
            inst,
            func: *handler_func,
            data: *handler_data,
        })?;
        board::Radio::<B>::enable().map_err(|_| Trap)?;
        api::register::Results {}
    };
    call.reply(results);
}

fn unregister<B: Board>(mut call: SchedulerCall<B, api::unregister::Sig>) {
    let results = try {
        board::Radio::<B>::disable().map_err(|_| Trap)?;
        call.scheduler().disable_event(Key::Received.into())?;
        api::unregister::Results {}
    };
    call.reply(results);
}

fn read<B: Board>(mut call: SchedulerCall<B, api::read::Sig>) {
    let api::read::Params { ptr, len } = call.read();
    let scheduler = call.scheduler();
    let memory = scheduler.applet.memory();
    let results = try {
        let output = memory.get_mut(*ptr, *len)?;
        let len = match board::Radio::<B>::read(output) {
            Ok(len) => (len as u32).into(),
            Err(_) => u32::MAX.into(),
        };
        api::read::Results { len }
    };
    call.reply(results);
}
