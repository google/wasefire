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
use crate::{Failure, Scheduler, SchedulerCall};

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
        Api::Serial(call) => or_fail!("board-api-platform", serial(call)),
        #[cfg(feature = "applet-api-platform")]
        Api::Version(call) => or_fail!("board-api-platform", version(call)),
        #[cfg(feature = "applet-api-platform")]
        Api::Reboot(call) => or_fail!("board-api-platform", reboot(call)),
    }
}

#[cfg(feature = "board-api-platform")]
fn serial<B: Board>(mut call: SchedulerCall<B, api::serial::Sig>) {
    let api::serial::Params { ptr } = call.read();
    let result = alloc_bytes(call.scheduler(), *ptr, &board::Platform::<B>::serial());
    call.reply(result);
}

#[cfg(feature = "board-api-platform")]
fn version<B: Board>(mut call: SchedulerCall<B, api::version::Sig>) {
    let api::version::Params { ptr } = call.read();
    let result = alloc_bytes(call.scheduler(), *ptr, &board::Platform::<B>::version());
    call.reply(result);
}

#[cfg(feature = "board-api-platform")]
fn reboot<B: Board>(call: SchedulerCall<B, api::reboot::Sig>) {
    let api::reboot::Params {} = call.read();
    call.reply(board::Platform::<B>::reboot().map_err(|x| x.into()));
}

#[cfg(feature = "board-api-platform")]
fn alloc_bytes<B: Board>(
    scheduler: &mut Scheduler<B>, ptr_ptr: u32, data: &[u8],
) -> Result<u32, Failure> {
    if data.is_empty() {
        return Ok(0);
    }
    let mut memory = scheduler.applet.memory();
    let len = data.len() as u32;
    let ptr = memory.alloc(len, 1)?;
    memory.get_mut(ptr, len)?.copy_from_slice(data);
    memory.get_mut(ptr_ptr, 4)?.copy_from_slice(&ptr.to_le_bytes());
    Ok(len)
}
