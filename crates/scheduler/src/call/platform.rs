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

#[cfg(feature = "board-api-platform")]
use wasefire_applet_api::platform as api;
use wasefire_applet_api::platform::Api;
#[cfg(feature = "board-api-platform")]
use wasefire_board_api as board;
#[cfg(feature = "board-api-platform")]
use wasefire_board_api::platform::Api as _;
use wasefire_board_api::Api as Board;

#[cfg(feature = "board-api-platform")]
use crate::applet::store::MemoryApi;
use crate::DispatchSchedulerCall;
#[cfg(feature = "board-api-platform")]
use crate::SchedulerCall;

#[cfg(feature = "applet-api-platform-update")]
mod update;

pub fn process<B: Board>(call: Api<DispatchSchedulerCall<B>>) {
    match call {
        #[cfg(feature = "applet-api-platform-update")]
        Api::Update(call) => update::process(call),
        #[cfg(feature = "applet-api-platform")]
        Api::Version(call) => or_fail!("board-api-platform", version(call)),
        #[cfg(feature = "applet-api-platform")]
        Api::Reboot(call) => or_fail!("board-api-platform", reboot(call)),
    }
}

#[cfg(feature = "board-api-platform")]
fn version<B: Board>(mut call: SchedulerCall<B, api::version::Sig>) {
    let api::version::Params { ptr, len } = call.read();
    let scheduler = call.scheduler();
    let memory = scheduler.applet.memory();
    let result = try {
        let output = memory.get_mut(*ptr, *len)?;
        Ok(board::Platform::<B>::version(output) as u32)
    };
    call.reply(result);
}

#[cfg(feature = "board-api-platform")]
fn reboot<B: Board>(call: SchedulerCall<B, api::reboot::Sig>) {
    call.reply(Ok(board::Platform::<B>::reboot()));
}
