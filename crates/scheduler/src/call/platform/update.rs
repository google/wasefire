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
        Api::Metadata(call) => metadata(call),
        Api::Initialize(call) => initialize(call),
        Api::Process(call) => process_(call),
        Api::Finalize(call) => finalize(call),
    }
}

fn is_supported<B: Board>(call: SchedulerCall<B, api::is_supported::Sig>) {
    let api::is_supported::Params {} = call.read();
    let results = try {
        let supported = board::platform::Update::<B>::SUPPORT as u32;
        api::is_supported::Results { supported: supported.into() }
    };
    call.reply(results)
}

fn metadata<B: Board>(mut call: SchedulerCall<B, api::metadata::Sig>) {
    let api::metadata::Params { ptr: ptr_ptr, len: len_ptr } = call.read();
    let scheduler = call.scheduler();
    let mut memory = scheduler.applet.memory();
    let results = try {
        let res = match board::platform::Update::<B>::metadata() {
            Ok(metadata) => {
                let len = metadata.len() as u32;
                let ptr = memory.alloc(len, 1)?;
                memory.get_mut(ptr, len)?.copy_from_slice(&metadata);
                memory.get_mut(*ptr_ptr, 4)?.copy_from_slice(&ptr.to_le_bytes());
                memory.get_mut(*len_ptr, 4)?.copy_from_slice(&len.to_le_bytes());
                0.into()
            }
            Err(error) => error.into(),
        };
        api::metadata::Results { res }
    };
    call.reply(results);
}

fn initialize<B: Board>(call: SchedulerCall<B, api::initialize::Sig>) {
    let api::initialize::Params { dry_run } = call.read();
    let results = try {
        let dry_run = match *dry_run {
            0 => false,
            1 => true,
            _ => Err(Trap)?,
        };
        let res = board::platform::Update::<B>::initialize(dry_run).into();
        api::initialize::Results { res }
    };
    call.reply(results);
}

fn process_<B: Board>(mut call: SchedulerCall<B, api::process::Sig>) {
    let api::process::Params { ptr, len } = call.read();
    let scheduler = call.scheduler();
    let memory = scheduler.applet.memory();
    let results = try {
        let chunk = memory.get(*ptr, *len)?;
        let res = board::platform::Update::<B>::process(chunk).into();
        api::process::Results { res }
    };
    call.reply(results);
}

fn finalize<B: Board>(call: SchedulerCall<B, api::finalize::Sig>) {
    let api::finalize::Params {} = call.read();
    let results = try {
        let res = board::platform::Update::<B>::finalize().into();
        api::finalize::Results { res }
    };
    call.reply(results);
}
