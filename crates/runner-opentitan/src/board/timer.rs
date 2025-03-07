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

use wasefire_board_api::timer::{Api, Command, Event};
use wasefire_board_api::{Id, Support};
use wasefire_error::Error;

use crate::board::with_state;

pub enum Impl {}

pub struct State {
    deadline: [u64; Impl::SUPPORT],
    periodic: [u64; Impl::SUPPORT],
}

pub fn init() -> State {
    State { deadline: [u64::MAX; Impl::SUPPORT], periodic: [0; Impl::SUPPORT] }
}

impl Support<usize> for Impl {
    const SUPPORT: usize = 4;
}

impl Api for Impl {
    fn arm(timer: Id<Self>, command: &Command) -> Result<(), Error> {
        with_state(|state| {
            let duration = command.duration_ms as u64 * 1000;
            let periodic = if command.periodic { duration } else { 0 };
            let deadline = crate::time::uptime_us() + duration;
            state.timer.deadline[*timer] = deadline;
            state.timer.periodic[*timer] = periodic;
            trigger(state, true);
            Ok(())
        })
    }

    fn disarm(timer: Id<Self>) -> Result<(), Error> {
        with_state(|state| {
            state.timer.deadline[*timer] = u64::MAX;
            state.timer.periodic[*timer] = 0;
            trigger(state, true);
            Ok(())
        })
    }
}

fn trigger(state: &mut crate::board::State, mut do_refresh: bool) {
    let mut stable = false;
    while !stable {
        stable = true;
        if do_refresh {
            refresh(state);
        } else {
            do_refresh = true;
        }
        let now = crate::time::uptime_us();
        for id in 0 .. Impl::SUPPORT {
            if now < state.timer.deadline[id] {
                continue;
            }
            let timer = Id::new(id).unwrap();
            state.events.push(Event { timer }.into());
            if state.timer.periodic[id] == 0 {
                state.timer.deadline[id] = u64::MAX;
            } else {
                state.timer.deadline[id] += state.timer.periodic[id];
            }
            stable = false;
        }
    }
}

fn refresh(state: &mut crate::board::State) {
    let mut deadline = u64::MAX;
    for id in 0 .. Impl::SUPPORT {
        deadline = core::cmp::min(deadline, state.timer.deadline[id]);
    }
    crate::time::deadline_us(deadline);
}

pub fn interrupt() {
    with_state(|state| trigger(state, false));
}
