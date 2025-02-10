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

#[cfg(feature = "applet-api-platform")]
use wasefire_applet_api::platform as api;
use wasefire_applet_api::platform::Api;
#[cfg(feature = "applet-api-platform")]
use wasefire_board_api as board;
use wasefire_board_api::Api as Board;
#[cfg(feature = "applet-api-platform")]
use wasefire_board_api::platform::Api as _;

use crate::DispatchSchedulerCall;
#[cfg(feature = "applet-api-platform")]
use crate::SchedulerCall;
#[cfg(feature = "applet-api-platform")]
use crate::applet::store::MemoryApi;

#[cfg(feature = "applet-api-platform-protocol")]
mod protocol;
#[cfg(feature = "applet-api-platform-update")]
mod update;

pub fn process<B: Board>(call: Api<DispatchSchedulerCall<B>>) {
    match call {
        #[cfg(feature = "applet-api-platform-protocol")]
        Api::Protocol(call) => protocol::process(call),
        #[cfg(feature = "applet-api-platform-update")]
        Api::Update(call) => update::process(call),
        #[cfg(feature = "applet-api-platform")]
        Api::Serial(call) => serial(call),
        #[cfg(feature = "applet-api-platform")]
        Api::Version(call) => version(call),
        #[cfg(feature = "applet-api-platform")]
        Api::Reboot(call) => reboot(call),
    }
}

#[cfg(feature = "applet-api-platform")]
fn serial<B: Board>(mut call: SchedulerCall<B, api::serial::Sig>) {
    let api::serial::Params { ptr } = call.read();
    let result = try { call.memory().alloc_copy(*ptr, None, &board::Platform::<B>::serial())? };
    call.reply(result);
}

#[cfg(feature = "applet-api-platform")]
fn version<B: Board>(mut call: SchedulerCall<B, api::version::Sig>) {
    let api::version::Params { ptr } = call.read();
    let result = try { call.memory().alloc_copy(*ptr, None, &board::Platform::<B>::version())? };
    call.reply(result);
}

#[cfg(feature = "applet-api-platform")]
fn reboot<B: Board>(call: SchedulerCall<B, api::reboot::Sig>) {
    let api::reboot::Params {} = call.read();
    call.reply(board::Platform::<B>::reboot().map_err(|x| x.into()));
}
