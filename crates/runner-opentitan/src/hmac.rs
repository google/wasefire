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

use earlgrey::HMAC;
use wasefire_error::{CodeParam, Error};
use wasefire_sync::{Mutex, MutexGuard};

pub struct Hmac {
    _lock: MutexGuard<'static, ()>,
}

impl Hmac {
    pub fn start(key: Option<&[u8; 64]>) -> Result<Self, Error> {
        let lock = HW.try_lock().ok_or(error(Code::Busy))?;
        let mut cfg = HMAC.cfg().reset();
        cfg.hmac_en(key.is_some()).sha_en(true).digest_swap(true);
        if key.is_some() {
            cfg.key_length(8); // Key_512
        }
        cfg.digest_size(1); // SHA2_256
        cfg.reg.write();
        if let Some(key) = key {
            for (&src, dst) in key.array_chunks().zip(0 ..) {
                HMAC.key(dst).write_raw(u32::from_be_bytes(src));
            }
        }
        HMAC.cmd().reset().hash_start(true).reg.write();
        Ok(Hmac { _lock: lock })
    }

    pub fn update(&mut self, data: &[u8]) {
        let fifo = HMAC.msg_fifo(0).addr();
        let mut words = data.array_chunks();
        for &word in &mut words {
            let word = u32::from_le_bytes(word);
            wait_fifo();
            unsafe { (fifo as *mut u32).write_volatile(word) };
        }
        for &byte in words.remainder() {
            wait_fifo();
            unsafe { (fifo as *mut u8).write_volatile(byte) };
        }
    }

    pub fn finalize(self, out: &mut [u8; 32]) -> Result<(), Error> {
        HMAC.intr_state().reset().hmac_done(true).hmac_err(true).reg.write();
        while !HMAC.status().read().fifo_empty() {}
        HMAC.cmd().reset().hash_process(true).reg.write();
        loop {
            let state = HMAC.intr_state().read();
            if state.hmac_err() {
                return Err(error_hw());
            }
            if state.hmac_done() {
                break;
            }
        }
        for (dst, src) in out.array_chunks_mut().zip(0 ..) {
            *dst = HMAC.digest(src).read_raw().to_le_bytes();
        }
        Ok(())
    }
}

static HW: Mutex<()> = Mutex::new(());

fn wait_fifo() {
    while HMAC.status().read().fifo_full() {}
}

fn error(code: Code) -> Error {
    Error::new(crate::error::Space::Hmac, code)
}
fn error_hw() -> Error {
    let code = (HMAC.err_code().read_raw() & 0x3fff) as u16 | Code::Hardware as u16;
    Error::new(crate::error::Space::Hmac, code)
}
#[repr(u16)]
enum Code {
    Busy = 0x8001,
    // Hardware errors use 0xc001..=0xffff.
    Hardware = 0xc000,
}

impl From<Code> for u16 {
    fn from(value: Code) -> Self {
        value as u16
    }
}

impl CodeParam for Code {}
