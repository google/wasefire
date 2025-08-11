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

use wasefire_applet_api::fingerprint::matcher as api;
use wasefire_applet_api::fingerprint::matcher::Api;
use wasefire_board_api::Api as Board;
#[cfg(feature = "board-api-fingerprint-matcher")]
use wasefire_board_api::applet::MemoryExt as _;
#[cfg(feature = "board-api-fingerprint-matcher")]
use wasefire_board_api::fingerprint::matcher::Api as _;
#[cfg(feature = "board-api-fingerprint-matcher")]
use wasefire_board_api::{self as board, Support};

#[cfg(feature = "board-api-fingerprint-matcher")]
use crate::event::{Handler, fingerprint::matcher::Key};
use crate::{DispatchSchedulerCall, SchedulerCall};

pub fn process<B: Board>(call: Api<DispatchSchedulerCall<B>>) {
    match call {
        Api::IsSupported(call) => is_supported(call),
        Api::TemplateLength(call) => {
            or_fail!("board-api-fingerprint-matcher", template_length(call))
        }
        Api::Enroll(call) => or_fail!("board-api-fingerprint-matcher", enroll(call)),
        Api::AbortEnroll(call) => or_fail!("board-api-fingerprint-matcher", abort_enroll(call)),
        Api::Identify(call) => or_fail!("board-api-fingerprint-matcher", identify(call)),
        Api::AbortIdentify(call) => or_fail!("board-api-fingerprint-matcher", abort_identify(call)),
        Api::DeleteTemplate(call) => {
            or_fail!("board-api-fingerprint-matcher", delete_template(call))
        }
        Api::ListTemplates(call) => or_fail!("board-api-fingerprint-matcher", list_templates(call)),
    }
}

fn is_supported<B: Board>(call: SchedulerCall<B, api::is_supported::Sig>) {
    let api::is_supported::Params {} = call.read();
    call.reply(Ok(or_false!(
        "board-api-fingerprint-matcher",
        board::fingerprint::Matcher::<B>::SUPPORT
    ) as u32));
}

#[cfg(feature = "board-api-fingerprint-matcher")]
fn template_length<B: Board>(call: SchedulerCall<B, api::template_length::Sig>) {
    let api::template_length::Params {} = call.read();
    call.reply(Ok(board::fingerprint::Matcher::<B>::TEMPLATE_ID_SIZE as u32));
}

#[cfg(feature = "board-api-fingerprint-matcher")]
fn enroll<B: Board>(mut call: SchedulerCall<B, api::enroll::Sig>) {
    let api::enroll::Params { handler_step_func, handler_step_data, handler_func, handler_data } =
        call.read();
    let inst = call.inst();
    let applet = call.applet();
    let result = try {
        applet.enable(Handler {
            key: Key::EnrollStep.into(),
            inst,
            func: *handler_step_func,
            data: *handler_step_data,
        })?;
        applet.enable(Handler {
            key: Key::Enroll.into(),
            inst,
            func: *handler_func,
            data: *handler_data,
        })?;
        board::fingerprint::Matcher::<B>::start_enroll()?;
    };
    call.reply(result);
}

#[cfg(feature = "board-api-fingerprint-matcher")]
fn abort_enroll<B: Board>(mut call: SchedulerCall<B, api::abort_enroll::Sig>) {
    let api::abort_enroll::Params {} = call.read();
    let result = try {
        board::fingerprint::Matcher::<B>::abort_enroll()?;
        call.scheduler().disable_event(Key::Enroll.into())?;
        call.scheduler().disable_event(Key::EnrollStep.into())?;
    };
    call.reply(result);
}

#[cfg(feature = "board-api-fingerprint-matcher")]
fn identify<B: Board>(mut call: SchedulerCall<B, api::identify::Sig>) {
    let api::identify::Params { template, handler_func, handler_data } = call.read();
    let inst = call.inst();
    let applet = call.applet();
    let result = try {
        applet.enable(Handler {
            key: Key::Identify.into(),
            inst,
            func: *handler_func,
            data: *handler_data,
        })?;
        let memory = applet.memory();
        let len = board::fingerprint::Matcher::<B>::TEMPLATE_ID_SIZE as u32;
        let template = memory.get_opt(*template, len)?;
        board::fingerprint::Matcher::<B>::start_identify(template)?;
    };
    call.reply(result);
}

#[cfg(feature = "board-api-fingerprint-matcher")]
fn abort_identify<B: Board>(mut call: SchedulerCall<B, api::abort_identify::Sig>) {
    let api::abort_identify::Params {} = call.read();
    let result = try {
        board::fingerprint::Matcher::<B>::abort_identify()?;
        call.scheduler().disable_event(Key::Identify.into())?;
    };
    call.reply(result);
}

#[cfg(feature = "board-api-fingerprint-matcher")]
fn delete_template<B: Board>(mut call: SchedulerCall<B, api::delete_template::Sig>) {
    let api::delete_template::Params { template } = call.read();
    let memory = call.memory();
    let result = try {
        let len = board::fingerprint::Matcher::<B>::TEMPLATE_ID_SIZE as u32;
        let template = memory.get_opt(*template, len)?;
        board::fingerprint::Matcher::<B>::delete_template(template)?;
    };
    call.reply(result);
}

#[cfg(feature = "board-api-fingerprint-matcher")]
fn list_templates<B: Board>(mut call: SchedulerCall<B, api::list_templates::Sig>) {
    let api::list_templates::Params { templates } = call.read();
    let mut memory = call.memory();
    let result = try {
        let len = board::fingerprint::Matcher::<B>::TEMPLATE_ID_SIZE;
        let src = board::fingerprint::Matcher::<B>::list_templates()?;
        let cnt = (src.len() / len) as u32;
        memory.alloc_copy(*templates, None, &src)?;
        cnt
    };
    call.reply(result);
}
