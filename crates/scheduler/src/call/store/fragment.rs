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

use core::ops::Range;

use wasefire_applet_api::store::fragment::{self as api, Api};
use wasefire_board_api::Api as Board;
use wasefire_store::fragment;

use super::convert;
use crate::{DispatchSchedulerCall, SchedulerCall, Trap};

pub fn process<B: Board>(call: Api<DispatchSchedulerCall<B>>) {
    match call {
        Api::Insert(call) => insert(call),
        Api::Remove(call) => remove(call),
        Api::Find(call) => find(call),
    }
}

fn insert<B: Board>(mut call: SchedulerCall<B, api::insert::Sig>) {
    let api::insert::Params { keys, ptr, len } = call.read();
    let scheduler = call.scheduler();
    let memory = scheduler.applet.memory();
    let results = try {
        let keys = decode_keys(keys);
        let value = memory.get(*ptr, *len)?;
        let res = match fragment::write(&mut scheduler.store, &keys, value) {
            Ok(()) => 0.into(),
            Err(e) => convert(e).into(),
        };
        api::insert::Results { res }
    };
    call.reply(results);
}

fn remove<B: Board>(mut call: SchedulerCall<B, api::remove::Sig>) {
    let api::remove::Params { keys } = call.read();
    let res = match fragment::delete(&mut call.scheduler().store, &decode_keys(keys)) {
        Ok(()) => 0.into(),
        Err(e) => convert(e).into(),
    };
    call.reply(Ok(api::remove::Results { res }));
}

fn find<B: Board>(mut call: SchedulerCall<B, api::find::Sig>) {
    let api::find::Params { keys, ptr: ptr_ptr, len: len_ptr } = call.read();
    let scheduler = call.scheduler();
    let mut memory = scheduler.applet.memory();
    let results = try {
        let mut results = api::find::Results::default();
        match fragment::read(&mut scheduler.store, &decode_keys(keys)) {
            Ok(None) => (),
            Ok(Some(value)) => {
                let len = value.len() as u32;
                let ptr = memory.alloc(len, 1);
                if ptr == 0 {
                    // This API doesn't support failing allocation.
                    Err(Trap)?;
                }
                memory.get_mut(ptr, len)?.copy_from_slice(&value);
                memory.get_mut(*ptr_ptr, 4)?.copy_from_slice(&ptr.to_le_bytes());
                memory.get_mut(*len_ptr, 4)?.copy_from_slice(&len.to_le_bytes());
                results.res = 1.into();
            }
            Err(e) => results.res = convert(e).into(),
        }
        results
    };
    call.reply(results);
}

fn decode_keys(keys: u32) -> Range<usize> {
    (keys & 0xffff) as usize .. ((keys >> 16) & 0xffff) as usize
}
