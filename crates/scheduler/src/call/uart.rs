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

use wasefire_applet_api::uart::{self as api, Api};
#[cfg(feature = "board-api-uart")]
use wasefire_board_api::uart::{Api as _, Direction, Event};
use wasefire_board_api::Api as Board;
#[cfg(feature = "board-api-uart")]
use wasefire_board_api::{self as board, Id, Support};

#[cfg(feature = "board-api-uart")]
use crate::applet::store::MemoryApi;
#[cfg(feature = "board-api-uart")]
use crate::event::{uart::Key, Handler};
#[cfg(feature = "board-api-uart")]
use crate::Trap;
use crate::{DispatchSchedulerCall, SchedulerCall};

pub fn process<B: Board>(call: Api<DispatchSchedulerCall<B>>) {
    match call {
        Api::Count(call) => count(call),
        Api::SetBaudrate(call) => or_fail!("board-api-uart", set_baudrate(call)),
        Api::Start(call) => or_fail!("board-api-uart", start(call)),
        Api::Stop(call) => or_fail!("board-api-uart", stop(call)),
        Api::Read(call) => or_fail!("board-api-uart", read(call)),
        Api::Write(call) => or_fail!("board-api-uart", write(call)),
        Api::Register(call) => or_fail!("board-api-uart", register(call)),
        Api::Unregister(call) => or_fail!("board-api-uart", unregister(call)),
    }
}

fn count<B: Board>(call: SchedulerCall<B, api::count::Sig>) {
    let api::count::Params {} = call.read();
    #[cfg(feature = "board-api-uart")]
    let count = board::Uart::<B>::SUPPORT as u32;
    #[cfg(not(feature = "board-api-uart"))]
    let count = 0;
    call.reply(Ok(count));
}

#[cfg(feature = "board-api-uart")]
fn set_baudrate<B: Board>(call: SchedulerCall<B, api::set_baudrate::Sig>) {
    let api::set_baudrate::Params { uart, baudrate } = call.read();
    let result = try {
        let uart = Id::new(*uart as usize).map_err(|_| Trap)?;
        board::Uart::<B>::set_baudrate(uart, *baudrate as usize)?
    };
    call.reply(result);
}

#[cfg(feature = "board-api-uart")]
fn start<B: Board>(call: SchedulerCall<B, api::start::Sig>) {
    let api::start::Params { uart } = call.read();
    let result = try {
        let uart = Id::new(*uart as usize).map_err(|_| Trap)?;
        board::Uart::<B>::start(uart)?
    };
    call.reply(result);
}

#[cfg(feature = "board-api-uart")]
fn stop<B: Board>(call: SchedulerCall<B, api::stop::Sig>) {
    let api::stop::Params { uart } = call.read();
    let result = try {
        let uart = Id::new(*uart as usize).map_err(|_| Trap)?;
        board::Uart::<B>::stop(uart)?
    };
    call.reply(result);
}

#[cfg(feature = "board-api-uart")]
fn read<B: Board>(mut call: SchedulerCall<B, api::read::Sig>) {
    let api::read::Params { uart, ptr, len } = call.read();
    let applet = call.applet();
    let memory = applet.memory();
    let result = try {
        let uart = Id::new(*uart as usize).map_err(|_| Trap)?;
        let output = memory.get_mut(*ptr, *len)?;
        board::Uart::<B>::read(uart, output)? as u32
    };
    call.reply(result);
}

#[cfg(feature = "board-api-uart")]
fn write<B: Board>(mut call: SchedulerCall<B, api::write::Sig>) {
    let api::write::Params { uart, ptr, len } = call.read();
    let applet = call.applet();
    let memory = applet.memory();
    let result = try {
        let uart = Id::new(*uart as usize).map_err(|_| Trap)?;
        let input = memory.get(*ptr, *len)?;
        board::Uart::<B>::write(uart, input)? as u32
    };
    call.reply(result);
}

#[cfg(feature = "board-api-uart")]
fn register<B: Board>(mut call: SchedulerCall<B, api::register::Sig>) {
    let api::register::Params { uart, event, handler_func, handler_data } = call.read();
    let inst = call.inst();
    let applet = call.applet();
    let result = try {
        let uart = Id::new(*uart as usize).map_err(|_| Trap)?;
        let event = convert_event(uart, *event)?;
        applet.enable(Handler {
            key: Key::from(&event).into(),
            inst,
            func: *handler_func,
            data: *handler_data,
        })?;
        board::Uart::<B>::enable(uart, event.direction)?
    };
    call.reply(result);
}

#[cfg(feature = "board-api-uart")]
fn unregister<B: Board>(mut call: SchedulerCall<B, api::unregister::Sig>) {
    let api::unregister::Params { uart, event } = call.read();
    let result = try {
        let uart = Id::new(*uart as usize).map_err(|_| Trap)?;
        let event = convert_event(uart, *event)?;
        board::Uart::<B>::disable(uart, event.direction).map_err(|_| Trap)?;
        call.scheduler().disable_event(Key::from(&event).into())?;
    };
    call.reply(result);
}

#[cfg(feature = "board-api-uart")]
fn convert_event<B: Board>(uart: Id<board::Uart<B>>, event: u32) -> Result<Event<B>, Trap> {
    let direction = match api::Event::try_from(event).map_err(|_| Trap)? {
        api::Event::Read => Direction::Read,
        api::Event::Write => Direction::Write,
    };
    Ok(Event { uart, direction })
}
