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

#[cfg(feature = "board-api-usb-serial")]
use wasefire_applet_api::usb::serial as api;
use wasefire_applet_api::usb::serial::Api;
#[cfg(feature = "board-api-usb-serial")]
use wasefire_board_api as board;
#[cfg(feature = "board-api-usb-serial")]
use wasefire_board_api::usb::serial::{Api as _, Event};
use wasefire_board_api::Api as Board;

#[cfg(feature = "board-api-usb-serial")]
use crate::applet::store::MemoryApi;
#[cfg(feature = "board-api-usb-serial")]
use crate::event::{usb::serial::Key, Handler};
use crate::DispatchSchedulerCall;
#[cfg(feature = "board-api-usb-serial")]
use crate::{SchedulerCall, Trap};

pub fn process<B: Board>(call: Api<DispatchSchedulerCall<B>>) {
    match call {
        Api::Read(call) => or_fail!("board-api-usb-serial", read(call)),
        Api::Write(call) => or_fail!("board-api-usb-serial", write(call)),
        Api::Register(call) => or_fail!("board-api-usb-serial", register(call)),
        Api::Unregister(call) => or_fail!("board-api-usb-serial", unregister(call)),
        Api::Flush(call) => or_fail!("board-api-usb-serial", flush(call)),
    }
}

#[cfg(feature = "board-api-usb-serial")]
fn read<B: Board>(mut call: SchedulerCall<B, api::read::Sig>) {
    let api::read::Params { ptr, len } = call.read();
    let scheduler = call.scheduler();
    let memory = scheduler.applet.memory();
    let result = try {
        let output = memory.get_mut(*ptr, *len)?;
        board::usb::Serial::<B>::read(output)? as u32
    };
    call.reply(result);
}

#[cfg(feature = "board-api-usb-serial")]
fn write<B: Board>(mut call: SchedulerCall<B, api::write::Sig>) {
    let api::write::Params { ptr, len } = call.read();
    let scheduler = call.scheduler();
    let memory = scheduler.applet.memory();
    let result = try {
        let input = memory.get(*ptr, *len)?;
        board::usb::Serial::<B>::write(input)? as u32
    };
    call.reply(result);
}

#[cfg(feature = "board-api-usb-serial")]
fn register<B: Board>(mut call: SchedulerCall<B, api::register::Sig>) {
    let api::register::Params { event, handler_func, handler_data } = call.read();
    let inst = call.inst();
    let scheduler = call.scheduler();
    let result = try {
        let event = convert_event(*event)?;
        scheduler.applet.enable(Handler {
            key: Key::from(&event).into(),
            inst,
            func: *handler_func,
            data: *handler_data,
        })?;
        board::usb::Serial::<B>::enable(&event)?
    };
    call.reply(result);
}

#[cfg(feature = "board-api-usb-serial")]
fn unregister<B: Board>(mut call: SchedulerCall<B, api::unregister::Sig>) {
    let api::unregister::Params { event } = call.read();
    let scheduler = call.scheduler();
    let result = try {
        let event = convert_event(*event)?;
        board::usb::Serial::<B>::disable(&event).map_err(|_| Trap)?;
        scheduler.disable_event(Key::from(&event).into())?;
    };
    call.reply(result);
}

#[cfg(feature = "board-api-usb-serial")]
fn flush<B: Board>(call: SchedulerCall<B, api::flush::Sig>) {
    let api::flush::Params {} = call.read();
    let result = try { board::usb::Serial::<B>::flush()? };
    call.reply(result);
}

#[cfg(feature = "board-api-usb-serial")]
fn convert_event(event: u32) -> Result<Event, Trap> {
    Ok(match api::Event::try_from(event).map_err(|_| Trap)? {
        api::Event::Read => Event::Read,
        api::Event::Write => Event::Write,
    })
}
