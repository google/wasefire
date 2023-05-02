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

use wasefire_applet_api::crypto as crypto_api;
use wasefire_applet_api::crypto::hash::{self as api, Algorithm, Api};
use wasefire_board_api::crypto::sha256::Api as _;
use wasefire_board_api::crypto::Api as _;
use wasefire_board_api::Api as Board;

use crate::stores::HashContext;
use crate::{DispatchSchedulerCall, SchedulerCall, Trap};

pub fn process<B: Board>(call: Api<DispatchSchedulerCall<B>>) {
    match call {
        Api::IsSupported(call) => is_supported(call),
        Api::Initialize(call) => initialize(call),
        Api::Update(call) => update(call),
        Api::Finalize(call) => finalize(call),
    }
}

fn is_supported<B: Board>(mut call: SchedulerCall<B, api::is_supported::Sig>) {
    let api::is_supported::Params { algorithm } = call.read();
    let results = try {
        let supported = match Algorithm::try_from(*algorithm).map_err(|_| Trap)? {
            Algorithm::Sha256 => call.scheduler().board.crypto().sha256().is_supported(),
        };
        api::is_supported::Results { supported: (supported as u32).into() }
    };
    call.reply(results)
}

fn initialize<B: Board>(mut call: SchedulerCall<B, api::initialize::Sig>) {
    let api::initialize::Params { algorithm } = call.read();
    let scheduler = call.scheduler();
    let results = try {
        let context = match Algorithm::try_from(*algorithm).map_err(|_| Trap)? {
            Algorithm::Sha256 => HashContext::Sha256(
                scheduler.board.crypto().sha256().initialize().map_err(|_| Trap)?,
            ),
        };
        let id = scheduler.applet.hashes.insert(context)? as u32;
        api::initialize::Results { id: id.into() }
    };
    call.reply(results);
}

fn update<B: Board>(mut call: SchedulerCall<B, api::update::Sig>) {
    let api::update::Params { id, data, length } = call.read();
    let scheduler = call.scheduler();
    let memory = scheduler.applet.memory.get();
    let results = try {
        let data = memory.get(*data, *length)?;
        match scheduler.applet.hashes.get_mut(*id as usize)? {
            HashContext::Sha256(context) => {
                scheduler.board.crypto().sha256().update(context, data).map_err(|_| Trap)?;
            }
        }
        api::update::Results {}
    };
    call.reply(results);
}

fn finalize<B: Board>(mut call: SchedulerCall<B, api::finalize::Sig>) {
    let api::finalize::Params { id, digest } = call.read();
    let scheduler = call.scheduler();
    let memory = scheduler.applet.memory.get();
    let results = try {
        let res = match scheduler.applet.hashes.take(*id as usize)? {
            HashContext::Sha256(context) => {
                let digest = memory.get_array_mut::<32>(*digest)?;
                scheduler.board.crypto().sha256().finalize(context, digest)
            }
        };
        let res = match res {
            Ok(()) => 0.into(),
            Err(_) => crypto_api::Error::InvalidArgument.into(),
        };
        api::finalize::Results { res }
    };
    call.reply(results);
}
