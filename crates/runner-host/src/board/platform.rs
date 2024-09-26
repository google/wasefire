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

use std::borrow::Cow;

use data_encoding::HEXLOWER_PERMISSIVE;
use wasefire_board_api::platform::Api;
use wasefire_board_api::Error;
use wasefire_error::Code;

pub enum Impl {}

impl Api for Impl {
    #[cfg(any(feature = "tcp", feature = "unix"))]
    type Protocol = wasefire_protocol_tokio::Impl<pipe::Impl>;
    #[cfg(feature = "usb")]
    type Protocol = crate::board::usb::ProtocolImpl;

    type Update = UpdateImpl;

    fn serial() -> Cow<'static, [u8]> {
        from_hex(option_env!("WASEFIRE_HOST_SERIAL"))
    }

    fn version() -> Cow<'static, [u8]> {
        from_hex(option_env!("WASEFIRE_HOST_VERSION"))
    }

    fn reboot() -> Result<!, Error> {
        Err(Error::world(Code::NotImplemented))
    }
}

pub enum UpdateImpl {}
impl wasefire_board_api::Support<bool> for UpdateImpl {
    const SUPPORT: bool = false;
}
impl wasefire_board_api::platform::update::Api for UpdateImpl {
    fn metadata() -> Result<Box<[u8]>, Error> {
        Err(Error::world(Code::NotImplemented))
    }
    fn initialize(_dry_run: bool) -> Result<(), Error> {
        Err(Error::world(Code::NotImplemented))
    }
    fn process(_chunk: &[u8]) -> Result<(), Error> {
        Err(Error::world(Code::NotImplemented))
    }
    fn finalize() -> Result<(), Error> {
        Err(Error::world(Code::NotImplemented))
    }
}

fn from_hex(x: Option<&str>) -> Cow<'static, [u8]> {
    HEXLOWER_PERMISSIVE.decode(x.unwrap_or_default().as_bytes()).unwrap().into()
}

pub(crate) fn vendor(request: &[u8]) -> Result<Box<[u8]>, Error> {
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

#[cfg(any(feature = "tcp", feature = "unix"))]
mod pipe {
    use wasefire_error::Error;
    use wasefire_protocol_tokio::{HasPipe, Pipe};

    use crate::with_state;

    pub enum Impl {}

    impl HasPipe for Impl {
        fn with_pipe<R>(f: impl FnOnce(&mut Pipe) -> R) -> R {
            with_state(|state| f(&mut state.pipe))
        }

        fn vendor(request: &[u8]) -> Result<Box<[u8]>, Error> {
            super::vendor(request)
        }
    }
}
