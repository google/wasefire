// Copyright 2025 Google LLC
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

use wasefire_applet_api::fingerprint::sensor as api;
use wasefire_applet_api::fingerprint::sensor::Api;
use wasefire_board_api::Api as Board;
#[cfg(feature = "board-api-fingerprint-sensor")]
use wasefire_board_api::fingerprint::sensor::Api as _;
#[cfg(feature = "board-api-fingerprint-sensor")]
use wasefire_board_api::{self as board, Support};

#[cfg(feature = "board-api-fingerprint-sensor")]
use crate::event::{Handler, fingerprint::sensor::Key};
use crate::{DispatchSchedulerCall, SchedulerCall};

pub fn process<B: Board>(call: Api<DispatchSchedulerCall<B>>) {
    match call {
        Api::IsSupported(call) => is_supported(call),
        Api::Capture(call) => or_fail!("board-api-fingerprint-sensor", capture(call)),
        Api::AbortCapture(call) => or_fail!("board-api-fingerprint-sensor", abort_capture(call)),
    }
}

fn is_supported<B: Board>(call: SchedulerCall<B, api::is_supported::Sig>) {
    let api::is_supported::Params {} = call.read();
    call.reply(Ok(or_false!(
        "board-api-fingerprint-sensor",
        board::fingerprint::Sensor::<B>::SUPPORT
    ) as u32));
}

#[cfg(feature = "board-api-fingerprint-sensor")]
fn capture<B: Board>(mut call: SchedulerCall<B, api::capture::Sig>) {
    let api::capture::Params { handler_func, handler_data } = call.read();
    let inst = call.inst();
    let applet = call.applet();
    let result = try {
        applet.enable(Handler {
            key: Key::Capture.into(),
            inst,
            func: *handler_func,
            data: *handler_data,
        })?;
        board::fingerprint::Sensor::<B>::start_capture()?;
    };
    call.reply(result);
}

#[cfg(feature = "board-api-fingerprint-sensor")]
fn abort_capture<B: Board>(mut call: SchedulerCall<B, api::abort_capture::Sig>) {
    let api::abort_capture::Params {} = call.read();
    let result = try {
        board::fingerprint::Sensor::<B>::abort_capture()?;
        call.scheduler().disable_event(Key::Capture.into())?;
    };
    call.reply(result);
}
