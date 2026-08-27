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

use wasefire_applet_api::store::Api;
#[cfg(feature = "board-api-store")]
use wasefire_applet_api::store::{self as api};
#[cfg(feature = "board-api-store")]
use wasefire_board_api as board;
use wasefire_board_api::Api as Board;
#[cfg(feature = "board-api-store")]
use wasefire_board_api::applet::{Memory as _, MemoryExt as _};
#[cfg(feature = "board-api-store")]
use wasefire_board_api::store::Api as _;

use crate::DispatchSchedulerCall;
#[cfg(feature = "board-api-store")]
use crate::SchedulerCall;

#[cfg(feature = "applet-api-store-fragment")]
mod fragment;

pub(super) fn process<B: Board>(call: Api<DispatchSchedulerCall<B>>) {
    match call {
        #[cfg(feature = "applet-api-store")]
        Api::MaxKey(call) => or_fail!("board-api-store", max_key(call)),
        #[cfg(feature = "applet-api-store")]
        Api::MaxLen(call) => or_fail!("board-api-store", max_len(call)),
        #[cfg(feature = "applet-api-store")]
        Api::Insert(call) => or_fail!("board-api-store", insert(call)),
        #[cfg(feature = "applet-api-store")]
        Api::Remove(call) => or_fail!("board-api-store", remove(call)),
        #[cfg(feature = "applet-api-store")]
        Api::Find(call) => or_fail!("board-api-store", find(call)),
        #[cfg(feature = "applet-api-store")]
        Api::Keys(call) => or_fail!("board-api-store", keys(call)),
        #[cfg(feature = "applet-api-store")]
        Api::Clear(call) => or_fail!("board-api-store", clear(call)),
        #[cfg(feature = "applet-api-store-fragment")]
        Api::Fragment(call) => fragment::process(call),
    }
}

#[cfg(feature = "board-api-store")]
fn max_key<B: Board>(call: SchedulerCall<B, api::max_key::Sig>) {
    let api::max_key::Params {} = call.read();
    call.reply(Ok(board::Store::<B>::MAX_KEY as u32));
}

#[cfg(feature = "board-api-store")]
fn max_len<B: Board>(call: SchedulerCall<B, api::max_len::Sig>) {
    let api::max_len::Params {} = call.read();
    call.reply(Ok(board::Store::<B>::MAX_LEN as u32));
}

#[cfg(feature = "board-api-store")]
fn insert<B: Board>(mut call: SchedulerCall<B, api::insert::Sig>) {
    let api::insert::Params { key, ptr, len } = call.read();
    let memory = call.memory();
    let result = try bikeshed _ {
        let value = memory.get(*ptr, *len)?;
        board::Store::<B>::insert(*key as usize, value)?
    };
    call.reply(result);
}

#[cfg(feature = "board-api-store")]
fn remove<B: Board>(call: SchedulerCall<B, api::remove::Sig>) {
    let api::remove::Params { key } = call.read();
    let res = try bikeshed _ { board::Store::<B>::remove(*key as usize)? };
    call.reply(res);
}

#[cfg(feature = "board-api-store")]
fn find<B: Board>(mut call: SchedulerCall<B, api::find::Sig>) {
    let api::find::Params { key, ptr: ptr_ptr, len: len_ptr } = call.read();
    let mut memory = call.memory();
    let result = try bikeshed _ {
        match board::Store::<B>::find(*key as usize)? {
            None => false,
            Some(value) => {
                memory.alloc_copy(*ptr_ptr, Some(*len_ptr), &value)?;
                true
            }
        }
    };
    call.reply(result);
}

#[cfg(feature = "board-api-store")]
fn keys<B: Board>(mut call: SchedulerCall<B, api::keys::Sig>) {
    let api::keys::Params { ptr: ptr_ptr } = call.read();
    let mut memory = call.memory();
    let result = try bikeshed _ {
        let keys = board::Store::<B>::keys()?;
        match keys {
            keys if keys.is_empty() => 0,
            keys => {
                let len = keys.len() as u32;
                let ptr = memory.alloc(2 * len, 2)?;
                memory.get_mut(ptr, 2 * len)?.copy_from_slice(bytemuck::cast_slice(&keys));
                memory.get_mut(*ptr_ptr, 4)?.copy_from_slice(&ptr.to_le_bytes());
                len
            }
        }
    };
    call.reply(result);
}

#[cfg(feature = "board-api-store")]
fn clear<B: Board>(call: SchedulerCall<B, api::clear::Sig>) {
    let api::clear::Params {} = call.read();
    let result = try bikeshed _ { board::Store::<B>::clear(0)? };
    call.reply(result);
}
