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

use alloc::borrow::Cow;

use earlgrey::LC_CTRL;
use wasefire_board_api::platform::Api;
use wasefire_common::platform::Side;
use wasefire_error::{Code, Error};
use wasefire_protocol::platform::SideInfo;
use wasefire_sync::Lazy;

mod update;

pub struct State {
    update: update::State,
}

pub fn init() -> State {
    let update = update::init();
    State { update }
}

pub enum Impl {}

impl Api for Impl {
    type Protocol = wasefire_protocol_usb::Impl<'static, crate::usb::Usb, crate::board::usb::Impl>;

    type Update = update::Impl;

    fn serial() -> Cow<'static, [u8]> {
        static SERIAL: Lazy<[u8; 8]> = Lazy::new(|| {
            let mut serial = [0; 8];
            // The individual device identification number is the second and third word of the
            // device identifier (bits 32 to 95 from the 256 bits).
            for (serial, index) in serial.chunks_mut(4).zip(1 ..) {
                serial.copy_from_slice(&LC_CTRL.device_id(index).read_raw().to_le_bytes());
            }
            serial
        });
        Cow::Borrowed(&*SERIAL)
    }

    fn running_side() -> Side {
        crate::flash::active()
    }

    fn running_info() -> SideInfo<'static> {
        static VERSION: Lazy<[u8; 20]> = Lazy::new(|| version(true));
        let name = Cow::default();
        let version = Cow::Borrowed(&VERSION[..]);
        SideInfo { name, version }
    }

    fn opposite_info() -> Result<SideInfo<'static>, Error> {
        let version = version(false);
        if version == [0xff; 20] {
            return Err(Error::world(Code::NotFound));
        }
        let name = Cow::default();
        let version = Cow::Owned(version.to_vec());
        Ok(SideInfo { name, version })
    }

    fn reboot() -> Result<!, Error> {
        crate::reboot()
    }
}

fn version(running: bool) -> [u8; 20] {
    let mut version = [0; 20];
    let manifest = if running { crate::manifest::active() } else { crate::manifest::inactive() };
    version[16 ..].copy_from_slice(&manifest.version_major);
    version[12 .. 16].copy_from_slice(&manifest.version_minor);
    version[8 .. 12].copy_from_slice(&manifest.security_version);
    version[.. 8].copy_from_slice(&manifest.timestamp);
    version.reverse();
    version
}
