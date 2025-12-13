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

#[cfg(feature = "internal-board-api-fingerprint")]
use wasefire_applet_api::fingerprint as api;
use wasefire_applet_api::fingerprint::Api;
use wasefire_board_api::Api as Board;
#[cfg(feature = "internal-board-api-fingerprint")]
use wasefire_board_api::{self as board, fingerprint::Api as _};
#[cfg(feature = "internal-board-api-fingerprint")]
use wasefire_error::{Code, Error};

use crate::DispatchSchedulerCall;
#[cfg(feature = "internal-board-api-fingerprint")]
use crate::SchedulerCall;
#[cfg(feature = "internal-board-api-fingerprint")]
use crate::event::{Handler, fingerprint::Key};

#[cfg(feature = "applet-api-fingerprint-matcher")]
mod matcher;
#[cfg(feature = "applet-api-fingerprint-sensor")]
mod sensor;

pub fn process<B: Board>(call: Api<DispatchSchedulerCall<B>>) {
    match call {
        #[cfg(feature = "applet-api-fingerprint-matcher")]
        Api::Matcher(call) => matcher::process(call),
        #[cfg(feature = "applet-api-fingerprint-sensor")]
        Api::Sensor(call) => sensor::process(call),
        Api::Register(call) => or_fail!("internal-board-api-fingerprint", register(call)),
        Api::Unregister(call) => or_fail!("internal-board-api-fingerprint", unregister(call)),
    }
}

#[cfg(feature = "internal-board-api-fingerprint")]
fn register<B: Board>(mut call: SchedulerCall<B, api::register::Sig>) {
    let api::register::Params { handler_func, handler_data } = call.read();
    let inst = call.inst();
    let result = try bikeshed _ {
        call.applet()
            .enable(Handler {
                key: Key::FingerDetected.into(),
                inst,
                func: *handler_func,
                data: *handler_data,
            })
            .map_err(|_| Error::user(Code::InvalidState))?;
        board::Fingerprint::<B>::enable()?;
    };
    call.reply(result);
}

#[cfg(feature = "internal-board-api-fingerprint")]
fn unregister<B: Board>(mut call: SchedulerCall<B, api::unregister::Sig>) {
    let api::unregister::Params {} = call.read();
    let result = try bikeshed _ {
        board::Fingerprint::<B>::disable()?;
        call.scheduler()
            .disable_event(Key::FingerDetected.into())
            .map_err(|_| Error::user(Code::InvalidState))?;
    };
    call.reply(result);
}
