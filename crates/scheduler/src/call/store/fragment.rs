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
use wasefire_board_api::{AppletMemory as _, AppletMemoryExt as _};
#[cfg(feature = "board-api-storage")]
use wasefire_store::fragment;

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
    let memory = scheduler.applet.get().unwrap().memory();
    let result = try {
        let keys = decode_keys(keys)?;
        let value = memory.get(*ptr, *len)?;
        fragment::write(&mut scheduler.store, &keys, value)?
    };
    call.reply(result);
}

#[cfg(feature = "board-api-storage")]
fn remove<B: Board>(mut call: SchedulerCall<B, api::remove::Sig>) {
    let api::remove::Params { keys } = call.read();
    let result = try { fragment::delete(&mut call.scheduler().store, &decode_keys(keys)?)? };
    call.reply(result);
}

#[cfg(feature = "board-api-storage")]
fn find<B: Board>(mut call: SchedulerCall<B, api::find::Sig>) {
    let api::find::Params { keys, ptr: ptr_ptr, len: len_ptr } = call.read();
    let scheduler = call.scheduler();
    let mut memory = scheduler.applet.get().unwrap().memory();
    let result = try {
        match fragment::read(&scheduler.store, &decode_keys(keys)?)? {
            None => false,
            Some(value) => {
                memory.alloc_copy(*ptr_ptr, Some(*len_ptr), &value)?;
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
