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

use board::button::Api;
use board::Error;

use crate::board::{Board, State};

impl Api for Board {
    fn count(&mut self) -> usize {
        1
    }

    fn enable(&mut self, button: usize) -> Result<(), Error> {
        if button != 0 {
            return Err(Error::User);
        }
        self.state.lock().unwrap().button = true;
        Ok(())
    }

    fn disable(&mut self, button: usize) -> Result<(), Error> {
        if button != 0 {
            return Err(Error::User);
        }
        self.state.lock().unwrap().button = false;
        Ok(())
    }
}

pub fn event(state: &mut State, pressed: Option<bool>) {
    if !state.button {
        return;
    }
    if pressed.unwrap_or(true) {
        let _ = state.sender.try_send(board::button::Event { button: 0, pressed: true }.into());
    }
    if !pressed.unwrap_or(false) {
        let _ = state.sender.try_send(board::button::Event { button: 0, pressed: false }.into());
    }
}
