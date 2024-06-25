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
use wasefire_board_api::Api as Board;
#[cfg(feature = "board-api-button")]
use wasefire_board_api::{self as board, button::Api as _, Id, Support};
#[cfg(feature = "board-api-button")]
use wasefire_error::{Code, Error};

#[cfg(feature = "board-api-button")]
use crate::event::{button::Key, Handler};
use crate::{DispatchSchedulerCall, SchedulerCall};

pub fn process<B: Board>(call: Api<DispatchSchedulerCall<B>>) {
    match call {
        Api::Count(call) => count(call),
        Api::Register(call) => or_trap!("board-api-button", register(call)),
        Api::Unregister(call) => or_trap!("board-api-button", unregister(call)),
    }
}

fn count<B: Board>(call: SchedulerCall<B, api::count::Sig>) {
    let api::count::Params {} = call.read();
    #[cfg(feature = "board-api-button")]
    let count = board::Button::<B>::SUPPORT as u32;
    #[cfg(not(feature = "board-api-button"))]
    let count = 0;
    call.reply(Ok(count));
}

#[cfg(feature = "board-api-button")]
fn register<B: Board>(mut call: SchedulerCall<B, api::register::Sig>) {
    let api::register::Params { button, handler_func, handler_data } = call.read();
    let inst = call.inst();
    let result = try {
        let button = Id::new(*button as usize)?;
        call.scheduler()
            .applet
            .enable(Handler {
                key: Key { button }.into(),
                inst,
                func: *handler_func,
                data: *handler_data,
            })
            .map_err(|_| Error::user(Code::InvalidState))?;
        board::Button::<B>::enable(button)?;
    };
    call.reply(result);
}

#[cfg(feature = "board-api-button")]
fn unregister<B: Board>(mut call: SchedulerCall<B, api::unregister::Sig>) {
    let api::unregister::Params { button } = call.read();
    let result = try {
        let button = Id::new(*button as usize)?;
        board::Button::<B>::disable(button)?;
        call.scheduler()
            .disable_event(Key { button }.into())
            .map_err(|_| Error::user(Code::InvalidState))?;
    };
    call.reply(result);
}
