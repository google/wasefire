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

use wasefire_applet_api::crypto::ccm::{self as api, Api};
use wasefire_board_api::crypto::aes128_ccm::Api as _;
use wasefire_board_api::crypto::Api as _;
use wasefire_board_api::Api as Board;

use crate::{DispatchSchedulerCall, SchedulerCall};

pub fn process<B: Board>(call: Api<DispatchSchedulerCall<B>>) {
    match call {
        Api::IsSupported(call) => is_supported(call),
        Api::Encrypt(call) => encrypt(call),
        Api::Decrypt(call) => decrypt(call),
    }
}

fn is_supported<B: Board>(mut call: SchedulerCall<B, api::is_supported::Sig>) {
    let api::is_supported::Params {} = call.read();
    let supported = call.scheduler().board.crypto().aes128_ccm().is_supported() as u32;
    call.reply(Ok(api::is_supported::Results { supported: supported.into() }))
}

fn encrypt<B: Board>(mut call: SchedulerCall<B, api::encrypt::Sig>) {
    let api::encrypt::Params { key, iv, len, clear, cipher } = call.read();
    let scheduler = call.scheduler();
    let memory = scheduler.applet.memory();
    let results = try {
        let key = memory.get(*key, 16)?;
        let iv = memory.get(*iv, 8)?;
        let clear = memory.get(*clear, *len)?;
        let cipher = memory.get_mut(*cipher, *len + 4)?;
        let res = match scheduler.board.crypto().aes128_ccm().encrypt(key, iv, clear, cipher) {
            Ok(()) => 0u32.into(),
            Err(_) => u32::MAX.into(),
        };
        api::encrypt::Results { res }
    };
    call.reply(results);
}

fn decrypt<B: Board>(mut call: SchedulerCall<B, api::decrypt::Sig>) {
    let api::decrypt::Params { key, iv, len, cipher, clear } = call.read();
    let scheduler = call.scheduler();
    let memory = scheduler.applet.memory();
    let results = try {
        let key = memory.get(*key, 16)?;
        let iv = memory.get(*iv, 8)?;
        let cipher = memory.get(*cipher, *len + 4)?;
        let clear = memory.get_mut(*clear, *len)?;
        let res = match scheduler.board.crypto().aes128_ccm().decrypt(key, iv, cipher, clear) {
            Ok(()) => 0u32.into(),
            Err(_) => u32::MAX.into(),
        };
        api::decrypt::Results { res }
    };
    call.reply(results);
}
