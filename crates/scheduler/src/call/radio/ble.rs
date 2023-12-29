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

use wasefire_applet_api::radio::ble::{self as api, Advertisement, Api};
use wasefire_board_api::radio::ble::{Api as _, Event};
use wasefire_board_api::{self as board, Api as Board};

use crate::applet::store::MemoryApi;
use crate::event::radio::ble::Key;
use crate::event::Handler;
use crate::{DispatchSchedulerCall, SchedulerCall, Trap};

pub fn process<B: Board>(call: Api<DispatchSchedulerCall<B>>) {
    match call {
        Api::Register(call) => register(call),
        Api::Unregister(call) => unregister(call),
        Api::ReadAdvertisement(call) => read_advertisement(call),
    }
}

fn register<B: Board>(mut call: SchedulerCall<B, api::register::Sig>) {
    let api::register::Params { event, handler_func, handler_data } = call.read();
    let inst = call.inst();
    let scheduler = call.scheduler();
    let results = try {
        let event = convert_event(event)?;
        scheduler.applet.enable(Handler {
            key: Key::from(&event).into(),
            inst,
            func: *handler_func,
            data: *handler_data,
        })?;
        board::radio::Ble::<B>::enable(&event).map_err(|_| Trap)?;
        api::register::Results {}
    };
    call.reply(results);
}

fn unregister<B: Board>(mut call: SchedulerCall<B, api::unregister::Sig>) {
    let api::unregister::Params { event } = call.read();
    let scheduler = call.scheduler();
    let results = try {
        let event = convert_event(event)?;
        board::radio::Ble::<B>::disable(&event).map_err(|_| Trap)?;
        scheduler.disable_event(Key::from(&event).into())?;
        api::unregister::Results {}
    };
    call.reply(results);
}

fn read_advertisement<B: Board>(mut call: SchedulerCall<B, api::read_advertisement::Sig>) {
    let api::read_advertisement::Params { ptr } = call.read();
    let scheduler = call.scheduler();
    let memory = scheduler.applet.memory();
    let results = try {
        let packet = memory.from_bytes_mut::<Advertisement>(*ptr)?;
        let res = board::radio::Ble::<B>::read_advertisement(packet).map(|x| x as u32).into();
        api::read_advertisement::Results { res }
    };
    call.reply(results);
}

fn convert_event(event: u32) -> Result<Event, Trap> {
    Ok(match api::Event::try_from(event)? {
        api::Event::Advertisement => Event::Advertisement,
    })
}
