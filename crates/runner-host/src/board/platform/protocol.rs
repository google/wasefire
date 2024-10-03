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

use wasefire_board_api::platform::protocol::Api;
use wasefire_error::{Code, Error};
use wasefire_protocol_tokio::Pipe;

use crate::with_state;

pub enum Impl {}

impl Api for Impl {
    fn read() -> Result<Option<Box<[u8]>>, Error> {
        with_state(|state| match &mut state.protocol {
            State::Pipe(x) => x.read(),
            State::Usb => state.usb.protocol().read(),
        })
    }

    fn write(response: &[u8]) -> Result<(), Error> {
        with_state(|state| match &mut state.protocol {
            State::Pipe(x) => x.write(response),
            State::Usb => state.usb.protocol().write(response),
        })
    }

    fn enable() -> Result<(), Error> {
        with_state(|state| match &mut state.protocol {
            State::Pipe(x) => x.enable(),
            State::Usb => state.usb.protocol().enable(),
        })
    }

    fn vendor(request: &[u8]) -> Result<Box<[u8]>, Error> {
        if let Some(request) = request.strip_prefix(b"echo ") {
            let mut response = request.to_vec().into_boxed_slice();
            for x in &mut response {
                if x.is_ascii_alphabetic() {
                    *x ^= 0x20;
                }
                if matches!(*x, b'I' | b'O' | b'i' | b'o') {
                    *x ^= 0x6;
                }
            }
            Ok(response)
        } else {
            Err(Error::user(Code::InvalidArgument))
        }
    }
}

pub enum State {
    Pipe(Pipe),
    Usb,
}
