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

use wasefire_applet_api::crypto::gcm::{self as api, Api, Support};
use wasefire_board_api::crypto::aead::Api as _;
use wasefire_board_api::{self as board, Api as Board, Support as _};

use crate::{DispatchSchedulerCall, SchedulerCall};

pub fn process<B: Board>(call: Api<DispatchSchedulerCall<B>>) {
    match call {
        Api::Support(call) => support(call),
        Api::Encrypt(call) => encrypt(call),
        Api::Decrypt(call) => decrypt(call),
    }
}

fn support<B: Board>(call: SchedulerCall<B, api::support::Sig>) {
    let api::support::Params {} = call.read();
    let support = board::crypto::Aes256Gcm::<B>::SUPPORT;
    let support = (support.no_copy as u32) << Support::NoCopy as u32
        | (support.in_place_no_copy as u32) << Support::InPlaceNoCopy as u32;
    call.reply(Ok(api::support::Results { support: support.into() }))
}

fn encrypt<B: Board>(mut call: SchedulerCall<B, api::encrypt::Sig>) {
    let api::encrypt::Params { key, iv, aad, aad_len, length, clear, cipher, tag } = call.read();
    let scheduler = call.scheduler();
    let memory = scheduler.applet.memory();
    let results = try {
        let key = memory.get_array::<32>(*key)?.into();
        let iv = memory.get_array::<12>(*iv)?.into();
        let aad = memory.get(*aad, *aad_len)?;
        let clear = memory.get_opt(*clear, *length)?;
        let cipher = memory.get_mut(*cipher, *length)?;
        let tag = memory.get_array_mut::<16>(*tag)?.into();
        let res = match board::crypto::Aes256Gcm::<B>::encrypt(key, iv, aad, clear, cipher, tag) {
            Ok(()) => 0u32.into(),
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
        let key = memory.get_array::<32>(*key)?.into();
        let iv = memory.get_array::<12>(*iv)?.into();
        let aad = memory.get(*aad, *aad_len)?;
        let tag = memory.get_array::<16>(*tag)?.into();
        let cipher = memory.get_opt(*cipher, *length)?;
        let clear = memory.get_mut(*clear, *length)?;
        let res = match board::crypto::Aes256Gcm::<B>::decrypt(key, iv, aad, cipher, tag, clear) {
            Ok(()) => 0u32.into(),
            Err(_) => u32::MAX.into(),
        };
        api::decrypt::Results { res }
    };
    call.reply(results);
}
