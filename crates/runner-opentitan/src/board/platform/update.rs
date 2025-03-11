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

use wasefire_board_api::Supported;
use wasefire_board_api::platform::update::Api;
use wasefire_common::platform::Side;
use wasefire_error::{Code, Error};
use wasefire_logger as log;

use crate::board::platform::version;
use crate::board::storage::Linear;
use crate::board::with_state;

pub struct State {
    storage: Option<Linear>,
}

pub fn init() -> State {
    State { storage: None }
}

pub enum Impl {}

impl Supported for Impl {}

impl Api for Impl {
    fn initialize(dry_run: bool) -> Result<(), Error> {
        with_state(|state| {
            let writing = Linear::start(dry_run, &state.storage.other)?;
            state.platform.update.storage = Some(writing);
            Ok(())
        })
    }

    fn process(chunk: &[u8]) -> Result<(), Error> {
        with_state(|state| {
            let Some(writing) = &mut state.platform.update.storage else {
                return Err(Error::user(Code::InvalidState));
            };
            writing.write(&state.storage.other, chunk)
        })
    }

    fn finalize() -> Result<(), Error> {
        with_state(|state| {
            let Some(writing) = state.platform.update.storage.take() else {
                return Err(Error::user(Code::InvalidState));
            };
            if writing.flush(&state.storage.other)?.is_some() { reboot() } else { Ok(()) }
        })
    }
}

fn reboot() -> ! {
    let next = crate::flash::active().opposite();
    let primary = if version(true) < version(false) { next.opposite() } else { next };
    log::info!("Rebooting to side {} (primary={})", next, primary);
    let mut msg = [0u8; 256];
    msg[32 ..][.. 4].copy_from_slice(b"BSVC");
    msg[36 ..][.. 4].copy_from_slice(b"NEXT");
    msg[40 ..][.. 4].copy_from_slice(&52u32.to_le_bytes());
    msg[44 ..][.. 4].copy_from_slice(slot(next));
    msg[48 ..][.. 4].copy_from_slice(slot(primary));
    let mut hash = crate::hmac::Hmac::start(None).unwrap();
    hash.update(&msg[32 .. 52]);
    hash.finalize((&mut msg[.. 32]).try_into().unwrap()).unwrap();
    msg[..32].reverse();
    for (addr, &word) in (0x40600008 ..).step_by(4).zip(msg.array_chunks()) {
        unsafe { (addr as *mut u32).write_volatile(u32::from_le_bytes(word)) };
    }
    crate::reboot();
}

fn slot(side: Side) -> &'static [u8; 4] {
    match side {
        Side::A => b"AA__",
        Side::B => b"__BB",
    }
}
