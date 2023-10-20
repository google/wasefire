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

use wasefire_board_api::led::Api;
use wasefire_board_api::{Error, Id, Support};

use crate::with_state;

pub enum Impl {}

impl Support<usize> for Impl {
    const SUPPORT: usize = 1;
}

impl Api for Impl {
    fn get(id: Id<Self>) -> Result<bool, Error> {
        assert_eq!(*id, 0);
        with_state(|state| Ok(state.led))
    }

    fn set(id: Id<Self>, on: bool) -> Result<(), Error> {
        assert_eq!(*id, 0);
        #[cfg(not(feature = "web"))]
        println!("Led is {}", if on { "on" } else { "off" });
        with_state(|state| {
            #[cfg(feature = "web")]
            state.web.set_led(on);
            state.led = on
        });
        Ok(())
    }
}
