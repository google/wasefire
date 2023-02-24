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

use wasefire_applet_api::clock::{self as api, Api};
use wasefire_board_api::timer::{Api as Bpi, Command};
use wasefire_board_api::Api as Board;

use crate::event::timer::Key;
use crate::event::Handler;
use crate::{DispatchSchedulerCall, Scheduler, SchedulerCall, Timer, Trap};

pub fn process<B: Board>(call: Api<DispatchSchedulerCall<B>>) {
    match call {
        Api::Allocate(call) => allocate(call),
        Api::Start(call) => start(call),
        Api::Stop(call) => stop(call),
        Api::Free(call) => free(call),
    }
}

fn allocate<B: Board>(mut call: SchedulerCall<B, api::allocate::Sig>) {
    let api::allocate::Params { handler_func, handler_data } = call.read();
    let inst = call.inst();
    let results = try {
        let timers = &mut call.scheduler().timers;
        let timer = timers.iter().position(|x| x.is_none()).ok_or(Trap)?;
        timers[timer] = Some(Timer {});
        call.scheduler().applet.enable(Handler {
            key: Key { timer }.into(),
            inst,
            func: *handler_func,
            data: *handler_data,
        })?;
        api::allocate::Results { id: (timer as u32).into() }
    };
    call.reply(results);
}

fn start<B: Board>(mut call: SchedulerCall<B, api::start::Sig>) {
    let api::start::Params { id, mode, duration_ms } = call.read();
    let timer = *id as usize;
    let results = try {
        get_timer(call.scheduler(), timer)?;
        let periodic = matches!(api::Mode::try_from(*mode)?, api::Mode::Periodic);
        let duration_ms = *duration_ms as usize;
        let command = Command { periodic, duration_ms };
        <B as Bpi>::arm(&mut call.scheduler().board, timer, &command).map_err(|_| Trap)?;
        api::start::Results {}
    };
    call.reply(results);
}

fn stop<B: Board>(mut call: SchedulerCall<B, api::stop::Sig>) {
    let api::stop::Params { id } = call.read();
    let timer = *id as usize;
    let results = try {
        get_timer(call.scheduler(), timer)?;
        <B as Bpi>::disarm(&mut call.scheduler().board, timer).map_err(|_| Trap)?;
        api::stop::Results {}
    };
    call.reply(results);
}

fn free<B: Board>(mut call: SchedulerCall<B, api::free::Sig>) {
    let api::free::Params { id } = call.read();
    let timer = *id as usize;
    let results = try {
        get_timer(call.scheduler(), timer)?;
        call.scheduler().disable_event(Key { timer }.into())?;
        call.scheduler().timers[timer] = None;
        api::free::Results {}
    };
    call.reply(results);
}

// TODO: Should also check that the timer belongs to the calling applet.
fn get_timer<B: Board>(scheduler: &mut Scheduler<B>, timer: usize) -> Result<&mut Timer, Trap> {
    match scheduler.timers.get_mut(timer) {
        Some(Some(x)) => Ok(x),
        _ => Err(Trap),
    }
}
