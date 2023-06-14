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

use wasefire_applet_api::usb::serial::{self as api, Api};
use wasefire_board_api::usb::serial::{Api as _, Event};
use wasefire_board_api::{self as board, Api as Board};

use crate::event::usb::serial::Key;
use crate::event::Handler;
use crate::{DispatchSchedulerCall, SchedulerCall, Trap};

pub fn process<B: Board>(call: Api<DispatchSchedulerCall<B>>) {
    match call {
        Api::Read(call) => read(call),
        Api::Write(call) => write(call),
        Api::Register(call) => register(call),
        Api::Unregister(call) => unregister(call),
        Api::Flush(call) => flush(call),
    }
}

fn read<B: Board>(mut call: SchedulerCall<B, api::read::Sig>) {
    let api::read::Params { ptr, len } = call.read();
    let scheduler = call.scheduler();
    let memory = scheduler.applet.memory();
    let results = try {
        let output = memory.get_mut(*ptr, *len)?;
        let len = match board::usb::Serial::<B>::read(output) {
            Ok(len) => (len as u32).into(),
            Err(_) => u32::MAX.into(),
        };
        api::read::Results { len }
    };
    call.reply(results);
}

fn write<B: Board>(mut call: SchedulerCall<B, api::write::Sig>) {
    let api::write::Params { ptr, len } = call.read();
    let scheduler = call.scheduler();
    let memory = scheduler.applet.memory();
    let results = try {
        let input = memory.get(*ptr, *len)?;
        let len = match board::usb::Serial::<B>::write(input) {
            Ok(len) => (len as u32).into(),
            Err(_) => u32::MAX.into(),
        };
        api::write::Results { len }
    };
    call.reply(results);
}

fn register<B: Board>(mut call: SchedulerCall<B, api::register::Sig>) {
    let api::register::Params { event, handler_func, handler_data } = call.read();
    let inst = call.inst();
    let scheduler = call.scheduler();
    let results = try {
        let event = convert_event(*event)?;
        scheduler.applet.enable(Handler {
            key: Key::from(&event).into(),
            inst,
            func: *handler_func,
            data: *handler_data,
        })?;
        board::usb::Serial::<B>::enable(&event).map_err(|_| Trap)?;
        api::register::Results {}
    };
    call.reply(results);
}

fn unregister<B: Board>(mut call: SchedulerCall<B, api::unregister::Sig>) {
    let api::unregister::Params { event } = call.read();
    let scheduler = call.scheduler();
    let results = try {
        let event = convert_event(*event)?;
        board::usb::Serial::<B>::disable(&event).map_err(|_| Trap)?;
        scheduler.disable_event(Key::from(&event).into())?;
        api::unregister::Results {}
    };
    call.reply(results);
}

fn flush<B: Board>(call: SchedulerCall<B, api::flush::Sig>) {
    let api::flush::Params {} = call.read();
    let results = try {
        let res = match board::usb::Serial::<B>::flush() {
            Ok(()) => 0.into(),
            Err(_) => u32::MAX.into(),
        };
        api::flush::Results { res }
    };
    call.reply(results);
}

fn convert_event(event: u32) -> Result<Event, Trap> {
    Ok(match api::Event::try_from(event)? {
        api::Event::Read => Event::Read,
        api::Event::Write => Event::Write,
    })
}
