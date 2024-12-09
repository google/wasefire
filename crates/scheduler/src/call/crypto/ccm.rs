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
use wasefire_board_api::Api as Board;
#[cfg(feature = "board-api-crypto-aes128-ccm")]
use wasefire_board_api::crypto::aead::{Api as _, Array};
#[cfg(feature = "board-api-crypto-aes128-ccm")]
use wasefire_board_api::{self as board, Support};

#[cfg(feature = "board-api-crypto-aes128-ccm")]
use crate::Trap;
#[cfg(feature = "board-api-crypto-aes128-ccm")]
use crate::applet::store::MemoryApi;
use crate::{DispatchSchedulerCall, SchedulerCall};

pub fn process<B: Board>(call: Api<DispatchSchedulerCall<B>>) {
    match call {
        Api::IsSupported(call) => is_supported(call),
        Api::Encrypt(call) => or_fail!("board-api-crypto-aes128-ccm", encrypt(call)),
        Api::Decrypt(call) => or_fail!("board-api-crypto-aes128-ccm", decrypt(call)),
    }
}

fn is_supported<B: Board>(call: SchedulerCall<B, api::is_supported::Sig>) {
    let api::is_supported::Params {} = call.read();
    #[cfg(feature = "board-api-crypto-aes128-ccm")]
    let supported = bool::from(board::crypto::Aes128Ccm::<B>::SUPPORT) as u32;
    #[cfg(not(feature = "board-api-crypto-aes128-ccm"))]
    let supported = 0;
    call.reply(Ok(supported))
}

#[cfg(feature = "board-api-crypto-aes128-ccm")]
fn encrypt<B: Board>(mut call: SchedulerCall<B, api::encrypt::Sig>) {
    let api::encrypt::Params { key, iv, len, clear, cipher } = call.read();
    let applet = call.applet();
    let memory = applet.memory();
    let result = try {
        ensure_support::<B>()?;
        let key = memory.get(*key, 16)?.into();
        let iv = expand_iv(memory.get(*iv, 8)?);
        let aad = &[0];
        let clear = Some(memory.get(*clear, *len)?);
        let (cipher, tag) = memory.get_mut(*cipher, *len + 4)?.split_at_mut(*len as usize);
        let tag = tag.into();
        board::crypto::Aes128Ccm::<B>::encrypt(key, &iv, aad, clear, cipher, tag)?
    };
    call.reply(result);
}

#[cfg(feature = "board-api-crypto-aes128-ccm")]
fn decrypt<B: Board>(mut call: SchedulerCall<B, api::decrypt::Sig>) {
    let api::decrypt::Params { key, iv, len, cipher, clear } = call.read();
    let applet = call.applet();
    let memory = applet.memory();
    let result = try {
        ensure_support::<B>()?;
        let key = memory.get(*key, 16)?.into();
        let iv = expand_iv(memory.get(*iv, 8)?);
        let aad = &[0];
        let (cipher, tag) = memory.get(*cipher, *len + 4)?.split_at(*len as usize);
        let cipher = Some(cipher);
        let tag = tag.into();
        let clear = memory.get_mut(*clear, *len)?;
        board::crypto::Aes128Ccm::<B>::decrypt(key, &iv, aad, cipher, tag, clear)?
    };
    call.reply(result);
}

#[cfg(feature = "board-api-crypto-aes128-ccm")]
fn expand_iv(iv: &[u8]) -> Array<typenum::U13> {
    core::array::from_fn(|i| i.checked_sub(5).map(|i| iv[i]).unwrap_or(0)).into()
}

#[cfg(feature = "board-api-crypto-aes128-ccm")]
fn ensure_support<B: Board>() -> Result<(), Trap> {
    match bool::from(board::crypto::Aes128Ccm::<B>::SUPPORT) {
        true => Ok(()),
        false => Err(Trap),
    }
}
