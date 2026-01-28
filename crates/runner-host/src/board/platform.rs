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
use wasefire_board_api::Error;
use wasefire_board_api::platform::Api;
use wasefire_common::platform::Side;
use wasefire_error::Code;
use wasefire_protocol::common::Name;
use wasefire_protocol::platform::SideInfo;

use crate::FLAGS;

pub mod protocol;
mod update;

pub enum Impl {}

impl Api for Impl {
    type Protocol = protocol::Impl;

    type Update = update::Impl;

    fn serial() -> Cow<'static, [u8]> {
        from_hex(FLAGS.serial.as_deref())
    }

    fn running_side() -> Side {
        Side::B
    }

    fn running_info() -> SideInfo<'static> {
        let name = String::from_utf8(from_hex(FLAGS.name.as_deref()).into_owned()).unwrap();
        let name = Name::new(name.into()).unwrap();
        let version = from_hex(FLAGS.version.as_deref());
        SideInfo { name, version }
    }

    fn opposite_info() -> Result<SideInfo<'static>, Error> {
        Err(Error::world(Code::NotEnough))
    }

    fn reboot() -> Result<!, Error> {
        crate::cleanup::shutdown(0)
    }
}

fn from_hex(x: Option<&str>) -> Cow<'static, [u8]> {
    HEXLOWER_PERMISSIVE.decode(x.unwrap_or_default().as_bytes()).unwrap().into()
}
