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

use wasefire_applet_api::crypto::gcm::{self as api, Api};
use wasefire_board_api::crypto::gcm::Api as _;
use wasefire_board_api::crypto::Api as _;
use wasefire_board_api::Api as Board;

use crate::{DispatchSchedulerCall, SchedulerCall};

pub fn process<B: Board>(call: Api<DispatchSchedulerCall<B>>) {
    match call {
        Api::Encrypt(call) => encrypt(call),
        Api::Decrypt(call) => decrypt(call),
    }
}

fn encrypt<B: Board>(mut call: SchedulerCall<B, api::encrypt::Sig>) {
    let api::encrypt::Params { key, iv, aad, aad_len, length, clear, cipher, tag } = call.read();
    let scheduler = call.scheduler();
    let memory = scheduler.applet.memory();
    let results = try {
        let key = memory.get_array::<32>(*key)?;
        let iv = memory.get_array::<12>(*iv)?;
        let aad = memory.get(*aad, *aad_len)?;
        let clear = memory.get(*clear, *length)?;
        let cipher = memory.get_mut(*cipher, *length)?;
        let tag = memory.get_array_mut::<16>(*tag)?;
        let res = match scheduler.board.crypto().gcm().encrypt(key, iv, aad, clear, cipher, tag) {
            Ok(()) => 0u32.into(),
            // TODO: The errors could be improved maybe.
            Err(_) => u32::MAX.into(),
        };
        api::encrypt::Results { res }
    };
    call.reply(results);
}

fn decrypt<B: Board>(mut call: SchedulerCall<B, api::decrypt::Sig>) {
    let api::decrypt::Params { key, iv, aad, aad_len, tag, length, cipher, clear } = call.read();
    let scheduler = call.scheduler();
    let memory = scheduler.applet.memory();
    let results = try {
        let key = memory.get_array::<32>(*key)?;
        let iv = memory.get_array::<12>(*iv)?;
        let aad = memory.get(*aad, *aad_len)?;
        let tag = memory.get_array::<16>(*tag)?;
        let cipher = memory.get(*cipher, *length)?;
        let clear = memory.get_mut(*clear, *length)?;
        let res = match scheduler.board.crypto().gcm().decrypt(key, iv, aad, tag, cipher, clear) {
            Ok(()) => 0u32.into(),
            // TODO: The errors could be improved maybe.
            Err(_) => u32::MAX.into(),
        };
        api::decrypt::Results { res }
    };
    call.reply(results);
}
