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

use wasefire_applet_api::platform::protocol as api;
use wasefire_applet_api::platform::protocol::Api;
use wasefire_board_api::Api as Board;

use crate::applet::store::MemoryApi;
use crate::event::Handler;
use crate::event::platform::protocol::Key;
use crate::{DispatchSchedulerCall, SchedulerCall};

pub fn process<B: Board>(call: Api<DispatchSchedulerCall<B>>) {
    match call {
        Api::Read(call) => read(call),
        Api::Write(call) => write(call),
        Api::Register(call) => register(call),
        Api::Unregister(call) => unregister(call),
    }
}

fn read<B: Board>(mut call: SchedulerCall<B, api::read::Sig>) {
    let api::read::Params { ptr: ptr_ptr, len: len_ptr } = call.read();
    let applet = call.applet();
    let result = try {
        match applet.get_request()? {
            None => false,
            Some(value) => {
                applet.memory().alloc_copy(*ptr_ptr, Some(*len_ptr), &value)?;
                true
            }
        }
    };
    call.reply(result);
}

fn write<B: Board>(mut call: SchedulerCall<B, api::write::Sig>) {
    let api::write::Params { ptr, len } = call.read();
    let result = try {
        let input = call.memory().get(*ptr, *len)?.into();
        crate::protocol::put_response(&mut call, input)?
    };
    call.reply(result);
}

fn register<B: Board>(mut call: SchedulerCall<B, api::register::Sig>) {
    let api::register::Params { handler_func, handler_data } = call.read();
    let inst = call.inst();
    let applet = call.applet();
    let result = try {
        // We don't need to enable the event at the board level because it's always enabled.
        applet.enable(Handler {
            key: Key::Request.into(),
            inst,
            func: *handler_func,
            data: *handler_data,
        })?
    };
    call.reply(result);
}

fn unregister<B: Board>(mut call: SchedulerCall<B, api::unregister::Sig>) {
    let api::unregister::Params {} = call.read();
    let result = try {
        // We only disable the applet handler because we still need to process non-applet requests.
        call.scheduler().disable_event(Key::Request.into())?
    };
    call.reply(result);
}
