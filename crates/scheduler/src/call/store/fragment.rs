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

use wasefire_applet_api::store::fragment::Api;
#[cfg(feature = "board-api-storage")]
use wasefire_applet_api::store::fragment::{self as api};
use wasefire_board_api::Api as Board;
#[cfg(feature = "board-api-storage")]
use wasefire_store::fragment;

#[cfg(feature = "board-api-storage")]
use super::convert;
#[cfg(feature = "board-api-storage")]
use crate::applet::store::MemoryApi;
use crate::DispatchSchedulerCall;
#[cfg(feature = "board-api-storage")]
use crate::SchedulerCall;

pub fn process<B: Board>(call: Api<DispatchSchedulerCall<B>>) {
    match call {
        Api::Insert(call) => or_fail!("board-api-storage", insert(call)),
        Api::Remove(call) => or_fail!("board-api-storage", remove(call)),
        Api::Find(call) => or_fail!("board-api-storage", find(call)),
    }
}

#[cfg(feature = "board-api-storage")]
fn insert<B: Board>(mut call: SchedulerCall<B, api::insert::Sig>) {
    let api::insert::Params { keys, ptr, len } = call.read();
    let scheduler = call.scheduler();
    let memory = scheduler.applet.memory();
    let result = try {
        let keys = decode_keys(keys)?;
        let value = memory.get(*ptr, *len)?;
        fragment::write(&mut scheduler.store, &keys, value).map_err(convert)?
    };
    call.reply(result);
}

#[cfg(feature = "board-api-storage")]
fn remove<B: Board>(mut call: SchedulerCall<B, api::remove::Sig>) {
    let api::remove::Params { keys } = call.read();
    let result = try {
        fragment::delete(&mut call.scheduler().store, &decode_keys(keys)?).map_err(convert)?
    };
    call.reply(result);
}

#[cfg(feature = "board-api-storage")]
fn find<B: Board>(mut call: SchedulerCall<B, api::find::Sig>) {
    let api::find::Params { keys, ptr: ptr_ptr, len: len_ptr } = call.read();
    let scheduler = call.scheduler();
    let mut memory = scheduler.applet.memory();
    let result = try {
        match fragment::read(&scheduler.store, &decode_keys(keys)?).map_err(convert)? {
            None => false,
            Some(value) => {
                let len = value.len() as u32;
                let ptr = memory.alloc(len, 1)?;
                memory.get_mut(ptr, len)?.copy_from_slice(&value);
                memory.get_mut(*ptr_ptr, 4)?.copy_from_slice(&ptr.to_le_bytes());
                memory.get_mut(*len_ptr, 4)?.copy_from_slice(&len.to_le_bytes());
                true
            }
        }
    };
    call.reply(result);
}

#[cfg(feature = "board-api-storage")]
fn decode_keys(keys: u32) -> Result<core::ops::Range<usize>, crate::Trap> {
    if keys & 0xf000f000 == 0 {
        Ok((keys & 0xffff) as usize .. ((keys >> 16) & 0xffff) as usize)
    } else {
        Err(crate::Trap)
    }
}
