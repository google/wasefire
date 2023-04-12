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

use wasefire_applet_api::scheduling::{self as api, Api};
use wasefire_board_api::Api as Board;

use crate::{DispatchSchedulerCall, SchedulerCall};

pub fn process<B: Board>(call: Api<DispatchSchedulerCall<B>>) {
    match call {
        Api::WaitForCallback(call) => wait_for_callback(call),
        Api::NumPendingCallbacks(call) => num_pending_callbacks(call),
        Api::Breakpoint(call) => breakpoint(call),
    }
}

fn wait_for_callback<B: Board>(mut call: SchedulerCall<B, api::wait_for_callback::Sig>) {
    let api::wait_for_callback::Params {} = call.read();
    if call.scheduler().process_event() {
        call.reply(Ok(api::wait_for_callback::Results {}));
    }
}

fn num_pending_callbacks<B: Board>(mut call: SchedulerCall<B, api::num_pending_callbacks::Sig>) {
    let api::num_pending_callbacks::Params {} = call.read();
    let count = (call.applet().len() as u32).into();
    call.reply(Ok(api::num_pending_callbacks::Results { count }));
}

fn breakpoint<B: Board>(mut call: SchedulerCall<B, api::breakpoint::Sig>) {
    let api::breakpoint::Params {} = call.read();
    call.scheduler().board.breakpoint();
    call.reply(Ok(api::breakpoint::Results {}));
}
