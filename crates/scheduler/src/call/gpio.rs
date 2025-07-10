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

use wasefire_applet_api::gpio::{self as api, Api};
use wasefire_board_api::Api as Board;
#[cfg(feature = "board-api-gpio")]
use wasefire_board_api::gpio::{Api as _, Event};
#[cfg(feature = "board-api-gpio")]
use wasefire_board_api::{self as board, Id, Support};
#[cfg(feature = "board-api-gpio")]
use wasefire_error::{Code, Error};

#[cfg(feature = "board-api-gpio")]
use crate::event::{Handler, gpio::Key};
use crate::{DispatchSchedulerCall, SchedulerCall};

pub fn process<B: Board>(call: Api<DispatchSchedulerCall<B>>) {
    match call {
        Api::Count(call) => count(call),
        Api::Configure(call) => or_fail!("board-api-gpio", configure(call)),
        Api::Read(call) => or_fail!("board-api-gpio", read(call)),
        Api::Write(call) => or_fail!("board-api-gpio", write(call)),
        Api::LastWrite(call) => or_fail!("board-api-gpio", last_write(call)),
        Api::Register(call) => or_fail!("board-api-gpio", register(call)),
        Api::Unregister(call) => or_fail!("board-api-gpio", unregister(call)),
    }
}

fn count<B: Board>(call: SchedulerCall<B, api::count::Sig>) {
    let api::count::Params {} = call.read();
    #[cfg(feature = "board-api-gpio")]
    let count = board::Gpio::<B>::SUPPORT as u32;
    #[cfg(not(feature = "board-api-gpio"))]
    let count = 0;
    call.reply(Ok(count));
}

#[cfg(feature = "board-api-gpio")]
fn configure<B: Board>(call: SchedulerCall<B, api::configure::Sig>) {
    let api::configure::Params { gpio, mode } = call.read();
    let result = try {
        let gpio = Id::new(*gpio as usize)?;
        let config = *bytemuck::checked::try_from_bytes(&mode.to_le_bytes())
            .map_err(|_| Error::user(Code::InvalidArgument))?;
        board::Gpio::<B>::configure(gpio, config)?
    };
    call.reply(result);
}

#[cfg(feature = "board-api-gpio")]
fn read<B: Board>(call: SchedulerCall<B, api::read::Sig>) {
    let api::read::Params { gpio } = call.read();
    let result = try {
        let gpio = Id::new(*gpio as usize)?;
        board::Gpio::<B>::read(gpio)?
    };
    call.reply(result);
}

#[cfg(feature = "board-api-gpio")]
fn write<B: Board>(call: SchedulerCall<B, api::write::Sig>) {
    let api::write::Params { gpio, val } = call.read();
    let result = try {
        let gpio = Id::new(*gpio as usize)?;
        let value = match *val {
            0 => false,
            1 => true,
            _ => Err(Error::user(Code::InvalidArgument))?,
        };
        board::Gpio::<B>::write(gpio, value)?
    };
    call.reply(result);
}

#[cfg(feature = "board-api-gpio")]
fn last_write<B: Board>(call: SchedulerCall<B, api::last_write::Sig>) {
    let api::last_write::Params { gpio } = call.read();
    let result = try {
        let gpio = Id::new(*gpio as usize)?;
        board::Gpio::<B>::last_write(gpio)?
    };
    call.reply(result);
}

#[cfg(feature = "board-api-gpio")]
fn register<B: Board>(mut call: SchedulerCall<B, api::register::Sig>) {
    let api::register::Params { gpio, event, handler_func, handler_data } = call.read();
    let inst = call.inst();
    let applet = call.applet();
    let result = try {
        let gpio = Id::new(*gpio as usize)?;
        let (falling, rising) = match api::Event::try_from(*event)? {
            api::Event::FallingEdge => (true, false),
            api::Event::RisingEdge => (false, true),
            api::Event::AnyChange => (true, true),
        };
        applet.enable(Handler {
            key: Key::from(&Event { gpio }).into(),
            inst,
            func: *handler_func,
            data: *handler_data,
        })?;
        board::Gpio::<B>::enable(gpio, falling, rising)?
    };
    call.reply(result);
}

#[cfg(feature = "board-api-gpio")]
fn unregister<B: Board>(mut call: SchedulerCall<B, api::unregister::Sig>) {
    let api::unregister::Params { gpio } = call.read();
    let result = try {
        let gpio = Id::new(*gpio as usize)?;
        board::Gpio::<B>::disable(gpio)?;
        call.scheduler().disable_event(Key::from(&Event { gpio }).into())?;
    };
    call.reply(result);
}
