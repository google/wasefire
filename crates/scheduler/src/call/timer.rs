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

use wasefire_applet_api::timer::{self as api, Api};
use wasefire_board_api::Api as Board;
#[cfg(feature = "board-api-timer")]
use wasefire_board_api::timer::{Api as _, Command};
#[cfg(feature = "board-api-timer")]
use wasefire_board_api::{self as board, Id};

#[cfg(feature = "board-api-timer")]
use crate::Trap;
#[cfg(feature = "board-api-timer")]
use crate::event::{Handler, timer::Key};
use crate::{DispatchSchedulerCall, SchedulerCall};
#[cfg(feature = "board-api-timer")]
use crate::{Scheduler, Timer};

pub fn process<B: Board>(call: Api<DispatchSchedulerCall<B>>) {
    match call {
        Api::Allocate(call) => allocate(call),
        Api::Start(call) => or_fail!("board-api-timer", start(call)),
        Api::Stop(call) => or_fail!("board-api-timer", stop(call)),
        Api::Free(call) => or_fail!("board-api-timer", free(call)),
    }
}

#[cfg(not(feature = "board-api-timer"))]
fn allocate<B: Board>(call: SchedulerCall<B, api::allocate::Sig>) {
    use wasefire_error::{Code, Error};
    call.reply_(Err(Error::world(Code::NotEnough).into()))
}

#[cfg(feature = "board-api-timer")]
fn allocate<B: Board>(mut call: SchedulerCall<B, api::allocate::Sig>) {
    let api::allocate::Params { handler_func, handler_data } = call.read();
    let inst = call.inst();
    let result = try {
        let timers = &mut call.scheduler().timers;
        let timer = timers.iter().position(|x| x.is_none()).ok_or(Trap)?;
        timers[timer] = Some(Timer {});
        call.applet().enable(Handler {
            key: Key { timer: Id::new(timer).unwrap() }.into(),
            inst,
            func: *handler_func,
            data: *handler_data,
        })?;
        timer as u32
    };
    call.reply(result);
}

#[cfg(feature = "board-api-timer")]
fn start<B: Board>(mut call: SchedulerCall<B, api::start::Sig>) {
    let api::start::Params { id, mode, duration_ms } = call.read();
    let timer = *id as usize;
    let result = try {
        let id = get_timer(call.scheduler(), timer)?;
        let periodic = matches!(api::Mode::try_from(*mode).map_err(|_| Trap)?, api::Mode::Periodic);
        let duration_ms = *duration_ms as usize;
        let command = Command { periodic, duration_ms };
        board::Timer::<B>::arm(id, &command)?
    };
    call.reply(result);
}

#[cfg(feature = "board-api-timer")]
fn stop<B: Board>(mut call: SchedulerCall<B, api::stop::Sig>) {
    let api::stop::Params { id } = call.read();
    let timer = *id as usize;
    let result = try {
        let id = get_timer(call.scheduler(), timer)?;
        board::Timer::<B>::disarm(id)?
    };
    call.reply(result);
}

#[cfg(feature = "board-api-timer")]
fn free<B: Board>(mut call: SchedulerCall<B, api::free::Sig>) {
    let api::free::Params { id } = call.read();
    let timer = *id as usize;
    let result = try {
        let timer = get_timer(call.scheduler(), timer)?;
        call.scheduler().disable_event(Key { timer }.into())?;
        call.scheduler().timers[*timer] = None;
    };
    call.reply(result);
}

// TODO: Should also check that the timer belongs to the calling applet.
#[cfg(feature = "board-api-timer")]
fn get_timer<B: Board>(
    scheduler: &Scheduler<B>, timer: usize,
) -> Result<Id<board::Timer<B>>, Trap> {
    let id = Id::new(timer).map_err(|_| Trap)?;
    if scheduler.timers[timer].is_none() {
        return Err(Trap);
    }
    Ok(id)
}
