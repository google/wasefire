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

use wasefire_board_api::button::{Api, Event};
use wasefire_board_api::{Error, Id, Support};

use crate::board::State;
use crate::with_state;

pub enum Impl {}

impl Support<usize> for Impl {
    const SUPPORT: usize = 1;
}

impl Api for Impl {
    fn enable(id: Id<Self>) -> Result<(), Error> {
        assert_eq!(*id, 0);
        with_state(|state| state.button = true);
        Ok(())
    }

    fn disable(id: Id<Self>) -> Result<(), Error> {
        assert_eq!(*id, 0);
        with_state(|state| state.button = false);
        Ok(())
    }
}

pub fn event(state: &mut State, pressed: Option<bool>) {
    if !state.button {
        return;
    }
    let button = Id::new(0).unwrap();
    if pressed.unwrap_or(true) {
        let _ = state.sender.try_send(Event { button, pressed: true }.into());
    }
    if !pressed.unwrap_or(false) {
        let _ = state.sender.try_send(Event { button, pressed: false }.into());
    }
}
