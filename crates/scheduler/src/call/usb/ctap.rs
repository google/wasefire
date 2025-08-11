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

#[cfg(feature = "board-api-usb-ctap")]
use wasefire_applet_api::usb::ctap as api;
use wasefire_applet_api::usb::ctap::Api;
#[cfg(feature = "board-api-usb-ctap")]
use wasefire_board_api as board;
use wasefire_board_api::Api as Board;
#[cfg(feature = "board-api-usb-ctap")]
use wasefire_board_api::AppletMemoryExt as _;
#[cfg(feature = "board-api-usb-ctap")]
use wasefire_board_api::usb::ctap::{Api as _, Event};

use crate::DispatchSchedulerCall;
#[cfg(feature = "board-api-usb-ctap")]
use crate::event::{Handler, usb::ctap::Key};
#[cfg(feature = "board-api-usb-ctap")]
use crate::{SchedulerCall, Trap};

pub fn process<B: Board>(call: Api<DispatchSchedulerCall<B>>) {
    match call {
        Api::Read(call) => or_fail!("board-api-usb-ctap", read(call)),
        Api::Write(call) => or_fail!("board-api-usb-ctap", write(call)),
        Api::Register(call) => or_fail!("board-api-usb-ctap", register(call)),
        Api::Unregister(call) => or_fail!("board-api-usb-ctap", unregister(call)),
    }
}

#[cfg(feature = "board-api-usb-ctap")]
fn read<B: Board>(mut call: SchedulerCall<B, api::read::Sig>) {
    let api::read::Params { ptr } = call.read();
    let applet = call.applet();
    let memory = applet.memory();
    let result = try {
        let output = memory.get_array_mut(*ptr)?;
        board::usb::Ctap::<B>::read(output)? as u32
    };
    call.reply(result);
}

#[cfg(feature = "board-api-usb-ctap")]
fn write<B: Board>(mut call: SchedulerCall<B, api::write::Sig>) {
    let api::write::Params { ptr } = call.read();
    let applet = call.applet();
    let memory = applet.memory();
    let result = try {
        let input = memory.get_array(*ptr)?;
        board::usb::Ctap::<B>::write(input)? as u32
    };
    call.reply(result);
}

#[cfg(feature = "board-api-usb-ctap")]
fn register<B: Board>(mut call: SchedulerCall<B, api::register::Sig>) {
    let api::register::Params { event, handler_func, handler_data } = call.read();
    let inst = call.inst();
    let applet = call.applet();
    let result = try {
        let event = convert_event(*event)?;
        applet.enable(Handler {
            key: Key::from(&event).into(),
            inst,
            func: *handler_func,
            data: *handler_data,
        })?;
        board::usb::Ctap::<B>::enable(&event)?
    };
    call.reply(result);
}

#[cfg(feature = "board-api-usb-ctap")]
fn unregister<B: Board>(mut call: SchedulerCall<B, api::unregister::Sig>) {
    let api::unregister::Params { event } = call.read();
    let result = try {
        let event = convert_event(*event)?;
        board::usb::Ctap::<B>::disable(&event).map_err(|_| Trap)?;
        call.scheduler().disable_event(Key::from(&event).into())?;
    };
    call.reply(result);
}

#[cfg(feature = "board-api-usb-ctap")]
fn convert_event(event: u32) -> Result<Event, Trap> {
    Ok(match api::Event::try_from(event).map_err(|_| Trap)? {
        api::Event::Read => Event::Read,
        api::Event::Write => Event::Write,
    })
}
