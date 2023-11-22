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

use wasefire_applet_api::store::{self as api, Api};
use wasefire_board_api::Api as Board;
use wasefire_store::StoreError;

use crate::applet::store::MemoryApi;
use crate::{DispatchSchedulerCall, SchedulerCall};

mod fragment;

pub fn process<B: Board>(call: Api<DispatchSchedulerCall<B>>) {
    match call {
        Api::Insert(call) => insert(call),
        Api::Remove(call) => remove(call),
        Api::Find(call) => find(call),
        Api::Fragment(call) => fragment::process(call),
    }
}

fn insert<B: Board>(mut call: SchedulerCall<B, api::insert::Sig>) {
    let api::insert::Params { key, ptr, len } = call.read();
    let scheduler = call.scheduler();
    let memory = scheduler.applet.memory();
    let results = try {
        let value = memory.get(*ptr, *len)?;
        let res = match scheduler.store.insert(*key as usize, value) {
            Ok(()) => 0.into(),
            Err(e) => convert(e).into(),
        };
        api::insert::Results { res }
    };
    call.reply(results);
}

fn remove<B: Board>(mut call: SchedulerCall<B, api::remove::Sig>) {
    let api::remove::Params { key } = call.read();
    let res = match call.scheduler().store.remove(*key as usize) {
        Ok(()) => 0.into(),
        Err(e) => convert(e).into(),
    };
    call.reply(Ok(api::remove::Results { res }));
}

fn find<B: Board>(mut call: SchedulerCall<B, api::find::Sig>) {
    #[cfg(feature = "multivalue")]
    let api::find::Params { key } = call.read();
    #[cfg(not(feature = "multivalue"))]
    let api::find::Params { key, ptr: ptr_ptr, len: len_ptr } = call.read();
    let scheduler = call.scheduler();
    let mut memory = scheduler.applet.memory();
    let results = try {
        let mut results = api::find::Results::default();
        match scheduler.store.find(*key as usize) {
            Ok(None) => (),
            Ok(Some(value)) => {
                let len = value.len() as u32;
                let ptr = memory.alloc(len, 1)?;
                memory.get_mut(ptr, len)?.copy_from_slice(&value);
                #[cfg(feature = "multivalue")]
                {
                    results.ptr = ptr.into();
                    results.len = len.into();
                }
                #[cfg(not(feature = "multivalue"))]
                {
                    memory.get_mut(*ptr_ptr, 4)?.copy_from_slice(&ptr.to_le_bytes());
                    memory.get_mut(*len_ptr, 4)?.copy_from_slice(&len.to_le_bytes());
                    results.res = 1.into();
                }
            }
            #[cfg(feature = "multivalue")]
            Err(e) => results.len = convert(e).into(),
            #[cfg(not(feature = "multivalue"))]
            Err(e) => results.res = convert(e).into(),
        }
        results
    };
    call.reply(results);
}

fn convert(err: StoreError) -> api::Error {
    match err {
        StoreError::InvalidArgument => api::Error::InvalidArgument,
        StoreError::NoCapacity => api::Error::NoCapacity,
        StoreError::NoLifetime => api::Error::NoLifetime,
        StoreError::StorageError => api::Error::StorageError,
        StoreError::InvalidStorage => api::Error::InvalidStorage,
    }
}
