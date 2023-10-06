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

use wasefire_applet_api::{self as api, Api};
use wasefire_board_api::Api as Board;

use crate::{DispatchSchedulerCall, SchedulerCall, Trap};

mod button;
mod clock;
mod crypto;
mod debug;
mod led;
mod rng;
mod scheduling;
mod store;
mod uart;
mod usb;

pub fn process<B: Board>(call: Api<DispatchSchedulerCall<B>>) {
    match call {
        Api::Button(call) => button::process(call),
        Api::Clock(call) => clock::process(call),
        Api::Crypto(call) => crypto::process(call),
        Api::Debug(call) => debug::process(call),
        Api::Led(call) => led::process(call),
        Api::Rng(call) => rng::process(call),
        Api::Scheduling(call) => scheduling::process(call),
        Api::Store(call) => store::process(call),
        Api::Syscall(call) => syscall(call),
        Api::Uart(call) => uart::process(call),
        Api::Usb(call) => usb::process(call),
    }
}

fn syscall<B: Board>(call: SchedulerCall<B, api::syscall::Sig>) {
    let api::syscall::Params { x1, x2, x3, x4 } = call.read();
    let results = try {
        let res = B::syscall(*x1, *x2, *x3, *x4).ok_or(Trap)?.into();
        api::syscall::Results { res }
    };
    call.reply(results);
}
