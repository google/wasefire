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

use alloc::vec;

use nrf52840_hal::ccm::CcmData;
use typenum::{U13, U16, U4};
use wasefire_board_api::crypto::aead::{AeadSupport, Api, Array};
use wasefire_board_api::{Error, Support};

use crate::with_state;

pub enum Impl {}

impl Support<AeadSupport> for Impl {
    const SUPPORT: AeadSupport = AeadSupport { no_copy: true, in_place_no_copy: true };
}

impl Api<U16, U13, U4> for Impl {
    fn encrypt(
        key: &Array<U16>, iv: &Array<U13>, aad: &[u8], clear: Option<&[u8]>, cipher: &mut [u8],
        tag: &mut Array<U4>,
    ) -> Result<(), Error> {
        if aad != [0] {
            return Err(Error::User);
        }
        let clear = match clear {
            Some(x) => x,
            None => cipher,
        };
        if 251 < clear.len() || cipher.len() != clear.len() {
            return Err(Error::User);
        }
        let len = clear.len();
        let mut ccm_data = CcmData::new((*key).into(), truncate_iv(iv));
        let mut clear_packet = vec![0; 3 + len];
        clear_packet[1] = len as u8;
        clear_packet[3 ..].copy_from_slice(clear);
        let mut cipher_packet = vec![0; 3 + len + 4];
        let mut scratch = vec![0; core::cmp::max(43, 16 + len)];
        match with_state(|state| {
            state.ccm.encrypt_packet(&mut ccm_data, &clear_packet, &mut cipher_packet, &mut scratch)
        }) {
            Ok(()) => {
                cipher.copy_from_slice(&cipher_packet[3 ..][.. len]);
                tag.copy_from_slice(&cipher_packet[3 + len ..]);
            }
            Err(_) => return Err(Error::World),
        }
        Ok(())
    }

    fn decrypt(
        key: &Array<U16>, iv: &Array<U13>, aad: &[u8], cipher: Option<&[u8]>, tag: &Array<U4>,
        clear: &mut [u8],
    ) -> Result<(), Error> {
        if aad != [0] {
            return Err(Error::User);
        }
        let cipher = match cipher {
            Some(x) => x,
            None => clear,
        };
        if 251 < clear.len() || cipher.len() != clear.len() {
            return Err(Error::User);
        }
        let len = clear.len();
        let mut ccm_data = CcmData::new((*key).into(), truncate_iv(iv));
        let mut cipher_packet = vec![0; 3 + len + 4];
        cipher_packet[1] = len as u8 + 4;
        cipher_packet[3 ..][.. len].copy_from_slice(cipher);
        cipher_packet[3 + len ..].copy_from_slice(tag);
        let mut clear_packet = vec![0; 3 + len];
        let mut scratch = vec![0; core::cmp::max(43, 16 + len + 4)];
        match with_state(|state| {
            state.ccm.decrypt_packet(&mut ccm_data, &mut clear_packet, &cipher_packet, &mut scratch)
        }) {
            Ok(()) => clear.copy_from_slice(&clear_packet[3 ..]),
            Err(_) => return Err(Error::World),
        }
        Ok(())
    }
}

fn truncate_iv(iv: &Array<U13>) -> [u8; 8] {
    core::array::from_fn(|i| iv[5 + i])
}
