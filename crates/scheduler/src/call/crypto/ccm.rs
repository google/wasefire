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
use wasefire_board_api::crypto::ccm::Api as Bpi;
use wasefire_board_api::Api as Board;

use crate::{DispatchSchedulerCall, SchedulerCall};

pub fn process<B: Board>(call: Api<DispatchSchedulerCall<B>>) {
    match call {
        Api::Encrypt(call) => encrypt(call),
        Api::Decrypt(call) => decrypt(call),
    }
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
        let res = match <B as Bpi>::encrypt(&mut scheduler.board, key, iv, clear, cipher) {
            Ok(()) => 0u32.into(),
            // TODO: The errors could be improved maybe.
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
        let res = match <B as Bpi>::decrypt(&mut scheduler.board, key, iv, cipher, clear) {
            Ok(()) => 0u32.into(),
            // TODO: The errors could be improved maybe.
            Err(_) => u32::MAX.into(),
        };
        api::decrypt::Results { res }
    };
    call.reply(results);
}
