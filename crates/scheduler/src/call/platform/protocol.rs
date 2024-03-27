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

#[cfg(feature = "board-api-platform-protocol")]
use wasefire_applet_api::platform::protocol as api;
use wasefire_applet_api::platform::protocol::Api;
use wasefire_board_api::Api as Board;

#[cfg(feature = "board-api-platform-protocol")]
use crate::applet::store::MemoryApi;
#[cfg(feature = "board-api-platform-protocol")]
use crate::event::{platform::protocol::Key, Handler};
use crate::DispatchSchedulerCall;
#[cfg(feature = "board-api-platform-protocol")]
use crate::SchedulerCall;

pub fn process<B: Board>(call: Api<DispatchSchedulerCall<B>>) {
    match call {
        Api::Read(call) => or_fail!("board-api-platform-protocol", read(call)),
        Api::Write(call) => or_fail!("board-api-platform-protocol", write(call)),
        Api::Register(call) => or_fail!("board-api-platform-protocol", register(call)),
        Api::Unregister(call) => or_fail!("board-api-platform-protocol", unregister(call)),
    }
}

#[cfg(feature = "board-api-platform-protocol")]
fn read<B: Board>(mut call: SchedulerCall<B, api::read::Sig>) {
    let api::read::Params { ptr: ptr_ptr, len: len_ptr } = call.read();
    let scheduler = call.scheduler();
    let result = try {
        match scheduler.applet.get_request() {
            Ok(None) => Ok(false),
            Ok(Some(value)) => {
                let mut memory = scheduler.applet.memory();
                let len = value.len() as u32;
                let ptr = memory.alloc(len, 1)?;
                memory.get_mut(ptr, len)?.copy_from_slice(&value);
                memory.get_mut(*ptr_ptr, 4)?.copy_from_slice(&ptr.to_le_bytes());
                memory.get_mut(*len_ptr, 4)?.copy_from_slice(&len.to_le_bytes());
                Ok(true)
            }
            Err(e) => Err(e),
        }
    };
    call.reply(result);
}

#[cfg(feature = "board-api-platform-protocol")]
fn write<B: Board>(mut call: SchedulerCall<B, api::write::Sig>) {
    let api::write::Params { ptr, len } = call.read();
    let scheduler = call.scheduler();
    let result = try {
        let memory = scheduler.applet.memory();
        let input = memory.get(*ptr, *len)?.into();
        crate::protocol::put_response(scheduler, input)
    };
    call.reply(result);
}

#[cfg(feature = "board-api-platform-protocol")]
fn register<B: Board>(mut call: SchedulerCall<B, api::register::Sig>) {
    let api::register::Params { handler_func, handler_data } = call.read();
    let inst = call.inst();
    let scheduler = call.scheduler();
    let result = try {
        // We don't need to enable the event at the board level because it's always enabled.
        scheduler.applet.enable(Handler {
            key: Key::Request.into(),
            inst,
            func: *handler_func,
            data: *handler_data,
        })?;
        Ok(())
    };
    call.reply(result);
}

#[cfg(feature = "board-api-platform-protocol")]
fn unregister<B: Board>(mut call: SchedulerCall<B, api::unregister::Sig>) {
    let api::unregister::Params {} = call.read();
    let scheduler = call.scheduler();
    let result = try {
        // We only disable the applet handler because we still need to process non-applet requests.
        scheduler.disable_event(Key::Request.into())?;
        Ok(())
    };
    call.reply(result);
}
