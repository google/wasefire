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

use wasefire_applet_api::platform::update::{self as api, Api};
use wasefire_board_api::platform::update::Api as _;
use wasefire_board_api::{self as board, Api as Board, Support};

use crate::applet::store::MemoryApi;
use crate::{DispatchSchedulerCall, SchedulerCall, Trap};

pub fn process<B: Board>(call: Api<DispatchSchedulerCall<B>>) {
    match call {
        Api::IsSupported(call) => is_supported(call),
        Api::Initialize(call) => initialize(call),
        Api::Process(call) => process_(call),
        Api::Finalize(call) => finalize(call),
    }
}

fn is_supported<B: Board>(call: SchedulerCall<B, api::is_supported::Sig>) {
    let api::is_supported::Params {} = call.read();
    let supported = board::platform::Update::<B>::SUPPORT as u32;
    call.reply(Ok(supported))
}

fn initialize<B: Board>(call: SchedulerCall<B, api::initialize::Sig>) {
    let api::initialize::Params { dry_run } = call.read();
    let result = try {
        let dry_run = match *dry_run {
            0 => false,
            1 => true,
            _ => Err(Trap)?,
        };
        board::platform::Update::<B>::initialize(dry_run)?
    };
    call.reply(result);
}

fn process_<B: Board>(mut call: SchedulerCall<B, api::process::Sig>) {
    let api::process::Params { ptr, len } = call.read();
    let applet = call.applet();
    let memory = applet.memory();
    let result = try {
        let chunk = memory.get(*ptr, *len)?;
        board::platform::Update::<B>::process(chunk)?
    };
    call.reply(result);
}

fn finalize<B: Board>(call: SchedulerCall<B, api::finalize::Sig>) {
    let api::finalize::Params {} = call.read();
    call.reply(try { board::platform::Update::<B>::finalize()? });
}
