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

#[cfg(feature = "board-api-storage")]
use alloc::vec::Vec;

use wasefire_applet_api::store::Api;
#[cfg(feature = "board-api-storage")]
use wasefire_applet_api::store::{self as api};
use wasefire_board_api::Api as Board;

use crate::DispatchSchedulerCall;
#[cfg(feature = "board-api-storage")]
use crate::SchedulerCall;
#[cfg(feature = "board-api-storage")]
use crate::applet::store::MemoryApi;

#[cfg(feature = "applet-api-store-fragment")]
mod fragment;

pub fn process<B: Board>(call: Api<DispatchSchedulerCall<B>>) {
    match call {
        #[cfg(feature = "applet-api-store")]
        Api::Insert(call) => or_fail!("board-api-storage", insert(call)),
        #[cfg(feature = "applet-api-store")]
        Api::Remove(call) => or_fail!("board-api-storage", remove(call)),
        #[cfg(feature = "applet-api-store")]
        Api::Find(call) => or_fail!("board-api-storage", find(call)),
        #[cfg(feature = "applet-api-store")]
        Api::Keys(call) => or_fail!("board-api-storage", keys(call)),
        #[cfg(feature = "applet-api-store")]
        Api::Clear(call) => or_fail!("board-api-storage", clear(call)),
        #[cfg(feature = "applet-api-store-fragment")]
        Api::Fragment(call) => fragment::process(call),
    }
}

#[cfg(feature = "board-api-storage")]
fn insert<B: Board>(mut call: SchedulerCall<B, api::insert::Sig>) {
    let api::insert::Params { key, ptr, len } = call.read();
    let scheduler = call.scheduler();
    let memory = scheduler.applet.get().unwrap().memory();
    let result = try {
        let value = memory.get(*ptr, *len)?;
        scheduler.store.insert(*key as usize, value)?
    };
    call.reply(result);
}

#[cfg(feature = "board-api-storage")]
fn remove<B: Board>(mut call: SchedulerCall<B, api::remove::Sig>) {
    let api::remove::Params { key } = call.read();
    let res = try { call.scheduler().store.remove(*key as usize)? };
    call.reply(res);
}

#[cfg(feature = "board-api-storage")]
fn find<B: Board>(mut call: SchedulerCall<B, api::find::Sig>) {
    let api::find::Params { key, ptr: ptr_ptr, len: len_ptr } = call.read();
    let scheduler = call.scheduler();
    let mut memory = scheduler.applet.get().unwrap().memory();
    let result = try {
        match scheduler.store.find(*key as usize)? {
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
fn keys<B: Board>(mut call: SchedulerCall<B, api::keys::Sig>) {
    let api::keys::Params { ptr: ptr_ptr } = call.read();
    let scheduler = call.scheduler();
    let mut memory = scheduler.applet.get().unwrap().memory();
    let result = try {
        let mut keys = Vec::new();
        for handle in scheduler.store.iter()? {
            keys.push(handle?.get_key() as u16);
        }
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

#[cfg(feature = "board-api-storage")]
fn clear<B: Board>(mut call: SchedulerCall<B, api::clear::Sig>) {
    let api::clear::Params {} = call.read();
    let result = try { call.scheduler().store.clear(0)? };
    call.reply(result);
}
