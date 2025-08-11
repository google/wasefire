// Copyright 2025 Google LLC
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

use alloc::vec::Vec;

use wasefire_board_api::applet::{Handlers, Memory};
use wasefire_board_api::vendor::Api;
#[cfg(feature = "test-vendor")]
use wasefire_board_api::vendor::Event;
use wasefire_board_api::{Error, Failure};

#[cfg(feature = "test-vendor")]
use crate::with_state;

pub enum Impl {}

impl Api for Impl {
    #[cfg(not(feature = "test-vendor"))]
    type Event = wasefire_board_api::vendor::NoEvent;
    #[cfg(feature = "test-vendor")]
    type Event = syscall_test::Event;

    #[cfg(not(feature = "test-vendor"))]
    type Key = ();
    #[cfg(feature = "test-vendor")]
    type Key = syscall_test::Key;

    fn key(event: &Self::Event) -> Self::Key {
        #[cfg(not(feature = "test-vendor"))]
        let (_, key) = (event, ());
        #[cfg(feature = "test-vendor")]
        let key = syscall_test::key(event);
        key
    }

    fn syscall(
        memory: impl Memory, handlers: impl Handlers<Self::Key>, x1: u32, x2: u32, x3: u32, x4: u32,
    ) -> Result<u32, Failure> {
        #[cfg(not(feature = "test-vendor"))]
        let _ = (memory, handlers);
        match (x1, x2, x3, x4) {
            #[cfg(feature = "test-vendor")]
            (0, _, _, _) => with_state(|state| {
                let push = |event| state.events.push(Event(event).into());
                syscall_test::syscall(memory, handlers, push, x2, x3, x4)
            }),
            #[cfg(feature = "gpio")]
            (0x80000000, x, y, z) => Ok(crate::board::gpio::syscall(x, y, z)?),
            _ => Err(Failure::TRAP),
        }
    }

    fn callback(
        memory: impl Memory, handlers: impl Handlers<Self::Key>, event: Self::Event,
        params: &mut Vec<u32>,
    ) {
        #[cfg(not(feature = "test-vendor"))]
        let _ = (memory, handlers, event, params);
        #[cfg(feature = "test-vendor")]
        syscall_test::callback(memory, handlers, event, params)
    }

    fn disable(key: Self::Key) -> Result<(), Error> {
        #[cfg(not(feature = "test-vendor"))]
        let () = key;
        #[cfg(feature = "test-vendor")]
        syscall_test::disable(key)?;
        Ok(())
    }
}
