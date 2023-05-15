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

use typenum::U13;
use wasefire_applet_api::crypto::ccm::{self as api, Api};
use wasefire_board_api::crypto::aead::{Api as _, Array};
use wasefire_board_api::{self as board, Api as Board, Support};

use crate::{DispatchSchedulerCall, SchedulerCall};

pub fn process<B: Board>(call: Api<DispatchSchedulerCall<B>>) {
    match call {
        Api::IsSupported(call) => is_supported(call),
        Api::Encrypt(call) => encrypt(call),
        Api::Decrypt(call) => decrypt(call),
    }
}

fn is_supported<B: Board>(call: SchedulerCall<B, api::is_supported::Sig>) {
    let api::is_supported::Params {} = call.read();
    let supported = bool::from(board::crypto::Aes128Ccm::<B>::SUPPORT) as u32;
    call.reply(Ok(api::is_supported::Results { supported: supported.into() }))
}

fn encrypt<B: Board>(mut call: SchedulerCall<B, api::encrypt::Sig>) {
    let api::encrypt::Params { key, iv, len, clear, cipher } = call.read();
    let scheduler = call.scheduler();
    let memory = scheduler.applet.memory();
    let results = try {
        let key = memory.get(*key, 16)?.into();
        let iv = expand_iv(memory.get(*iv, 8)?);
        let aad = &[0];
        let clear = Some(memory.get(*clear, *len)?);
        let (cipher, tag) = memory.get_mut(*cipher, *len + 4)?.split_at_mut(*len as usize);
        let tag = tag.into();
        let res = match board::crypto::Aes128Ccm::<B>::encrypt(key, &iv, aad, clear, cipher, tag) {
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
        let key = memory.get(*key, 16)?.into();
        let iv = expand_iv(memory.get(*iv, 8)?);
        let aad = &[0];
        let (cipher, tag) = memory.get(*cipher, *len + 4)?.split_at(*len as usize);
        let cipher = Some(cipher);
        let tag = tag.into();
        let clear = memory.get_mut(*clear, *len)?;
        let res = match board::crypto::Aes128Ccm::<B>::decrypt(key, &iv, aad, cipher, tag, clear) {
            Ok(()) => 0u32.into(),
            Err(_) => u32::MAX.into(),
        };
        api::decrypt::Results { res }
    };
    call.reply(results);
}

fn expand_iv(iv: &[u8]) -> Array<U13> {
    core::array::from_fn(|i| i.checked_sub(5).map(|i| iv[i]).unwrap_or(0)).into()
}
