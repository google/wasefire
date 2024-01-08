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
#[cfg(feature = "board-api-storage")]
use wasefire_applet_api::store::{self as api};
use wasefire_board_api::Api as Board;
#[cfg(feature = "board-api-storage")]
use wasefire_error::{Code, Error};
#[cfg(feature = "board-api-storage")]
use wasefire_store::StoreError;

#[cfg(feature = "board-api-storage")]
use crate::applet::store::MemoryApi;
use crate::DispatchSchedulerCall;
#[cfg(feature = "board-api-storage")]
use crate::SchedulerCall;

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
        #[cfg(feature = "applet-api-store-fragment")]
        Api::Fragment(call) => fragment::process(call),
    }
}

#[cfg(feature = "board-api-storage")]
fn insert<B: Board>(mut call: SchedulerCall<B, api::insert::Sig>) {
    let api::insert::Params { key, ptr, len } = call.read();
    let scheduler = call.scheduler();
    let memory = scheduler.applet.memory();
    let result = try {
        let value = memory.get(*ptr, *len)?;
        scheduler.store.insert(*key as usize, value).map_err(convert)
    };
    call.reply(result);
}

#[cfg(feature = "board-api-storage")]
fn remove<B: Board>(mut call: SchedulerCall<B, api::remove::Sig>) {
    let api::remove::Params { key } = call.read();
    let res = call.scheduler().store.remove(*key as usize).map_err(convert);
    call.reply(Ok(res));
}

#[cfg(feature = "board-api-storage")]
fn find<B: Board>(mut call: SchedulerCall<B, api::find::Sig>) {
    let api::find::Params { key, ptr: ptr_ptr, len: len_ptr } = call.read();
    let scheduler = call.scheduler();
    let mut memory = scheduler.applet.memory();
    let result = try {
        match scheduler.store.find(*key as usize) {
            Ok(None) => Ok(false),
            Ok(Some(value)) => {
                let len = value.len() as u32;
                let ptr = memory.alloc(len, 1)?;
                memory.get_mut(ptr, len)?.copy_from_slice(&value);
                memory.get_mut(*ptr_ptr, 4)?.copy_from_slice(&ptr.to_le_bytes());
                memory.get_mut(*len_ptr, 4)?.copy_from_slice(&len.to_le_bytes());
                Ok(true)
            }
            Err(e) => Err(convert(e)),
        }
    };
    call.reply(result);
}

#[cfg(feature = "board-api-storage")]
fn convert(err: StoreError) -> Error {
    match err {
        StoreError::InvalidArgument => Error::user(0),
        StoreError::NoCapacity | StoreError::NoLifetime => Error::user(Code::NotEnough),
        StoreError::StorageError => Error::world(0),
        StoreError::InvalidStorage => Error::world(Code::BadState),
    }
}
