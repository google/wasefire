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

use core::sync::atomic::Ordering::Relaxed;

use wasefire_board_api::rng::Api;
use wasefire_error::{Code, Error};
use wasefire_logger as log;
use wasefire_sync::AtomicBool;

use crate::crypto::drbg;

pub fn init() {
    match drbg::instantiate() {
        Ok(()) => INSTANTIATED.store(true, Relaxed),
        Err(error) => log::error!("Failed to initialize DRBG: {}", error),
    }
}

pub enum Impl {}

impl Api for Impl {
    fn fill_bytes(buffer: &mut [u8]) -> Result<(), Error> {
        match INSTANTIATED.load(Relaxed) {
            false => Err(Error::world(Code::InvalidState)),
            true => drbg::generate(buffer),
        }
    }
}

static INSTANTIATED: AtomicBool = AtomicBool::new(false);
