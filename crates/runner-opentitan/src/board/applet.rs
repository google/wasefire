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

use wasefire_board_api::applet::Api;
use wasefire_error::{Code, Error};

use crate::board::with_state;

mod install;

pub struct State {
    install: install::State,
}

pub fn init() -> State {
    let install = install::init();
    State { install }
}

pub enum Impl {}

impl Api for Impl {
    type Install = install::Impl;

    unsafe fn get() -> Result<&'static [u8], Error> {
        with_state(|state| {
            if state.applet.install.ongoing() {
                return Err(Error::user(Code::InvalidState));
            }
            let side = crate::flash::inactive() as usize;
            let content = unsafe { state.storage.applets[side].read() };
            let len = !u32::from_ne_bytes(content[content.len() - 4 ..].try_into().unwrap());
            content.get(.. len as usize).ok_or(Error::internal(Code::InvalidState))
        })
    }
}
