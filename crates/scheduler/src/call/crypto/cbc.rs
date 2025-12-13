// Copyright 2024 Google LLC
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

use wasefire_applet_api::crypto::cbc::{self as api, Api};
use wasefire_board_api::Api as Board;
#[cfg(feature = "board-api-crypto-aes256-cbc")]
use wasefire_board_api::applet::Memory as _;
#[cfg(feature = "board-api-crypto-aes256-cbc")]
use wasefire_board_api::crypto::cbc::Api as _;
#[cfg(feature = "board-api-crypto-aes256-cbc")]
use wasefire_board_api::{self as board, Support};

#[cfg(feature = "board-api-crypto-aes256-cbc")]
use crate::Trap;
use crate::{DispatchSchedulerCall, SchedulerCall};

pub fn process<B: Board>(call: Api<DispatchSchedulerCall<B>>) {
    match call {
        Api::IsSupported(call) => is_supported(call),
        Api::Encrypt(call) => or_fail!("board-api-crypto-aes256-cbc", encrypt(call)),
        Api::Decrypt(call) => or_fail!("board-api-crypto-aes256-cbc", decrypt(call)),
    }
}

fn is_supported<B: Board>(call: SchedulerCall<B, api::is_supported::Sig>) {
    let api::is_supported::Params {} = call.read();
    #[cfg(feature = "board-api-crypto-aes256-cbc")]
    let supported = board::crypto::Aes256Cbc::<B>::SUPPORT;
    #[cfg(not(feature = "board-api-crypto-aes256-cbc"))]
    let supported = 0;
    call.reply(Ok(supported))
}

#[cfg(feature = "board-api-crypto-aes256-cbc")]
fn encrypt<B: Board>(mut call: SchedulerCall<B, api::encrypt::Sig>) {
    let api::encrypt::Params { key, iv, ptr, len } = call.read();
    let memory = call.applet().memory();
    let result = try bikeshed _ {
        ensure_support::<B>()?;
        let key = memory.get(*key, 32)?.into();
        let iv = memory.get(*iv, 16)?.into();
        let blocks = memory.get_mut(*ptr, *len)?;
        board::crypto::Aes256Cbc::<B>::encrypt(key, iv, blocks)?
    };
    call.reply(result);
}

#[cfg(feature = "board-api-crypto-aes256-cbc")]
fn decrypt<B: Board>(mut call: SchedulerCall<B, api::decrypt::Sig>) {
    let api::decrypt::Params { key, iv, ptr, len } = call.read();
    let memory = call.applet().memory();
    let result = try bikeshed _ {
        ensure_support::<B>()?;
        let key = memory.get(*key, 32)?.into();
        let iv = memory.get(*iv, 16)?.into();
        let blocks = memory.get_mut(*ptr, *len)?;
        board::crypto::Aes256Cbc::<B>::decrypt(key, iv, blocks)?
    };
    call.reply(result);
}

#[cfg(feature = "board-api-crypto-aes256-cbc")]
fn ensure_support<B: Board>() -> Result<(), Trap> {
    match board::crypto::Aes256Cbc::<B>::SUPPORT {
        true => Ok(()),
        false => Err(Trap),
    }
}
