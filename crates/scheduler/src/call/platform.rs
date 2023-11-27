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

use wasefire_applet_api::platform::{self as api, Api};
use wasefire_board_api::platform::Api as _;
use wasefire_board_api::{self as board, Api as Board};

use crate::{DispatchSchedulerCall, SchedulerCall};

pub fn process<B: Board>(call: Api<DispatchSchedulerCall<B>>) {
    match call {
        Api::Reboot(call) => reboot(call),
    }
}

fn reboot<B: Board>(call: SchedulerCall<B, api::reboot::Sig>) {
    let api::reboot::Params {} = call.read();
    let res = match board::Platform::<B>::reboot() {
        Ok(x) => match x {},
        Err(_) => u32::MAX,
    };
    call.reply(Ok(api::reboot::Results { res: res.into() }));
}
