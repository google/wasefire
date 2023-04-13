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
use wasefire_board_api as board;

impl board::crypto::ccm::Api for &mut crate::tasks::Board {
    fn encrypt(
        &mut self, key: &[u8], iv: &[u8], clear: &[u8], cipher: &mut [u8],
    ) -> Result<(), board::Error> {
        let key = key.try_into().map_err(|_| board::Error::User)?;
        let iv = iv.try_into().map_err(|_| board::Error::User)?;
        if 251 < clear.len() || cipher.len() != clear.len() + 4 {
            return Err(board::Error::User);
        }
        let mut ccm_data = CcmData::new(key, iv);
        let mut clear_packet = vec![0; 3 + clear.len()];
        clear_packet[1] = clear.len() as u8;
        clear_packet[3 ..].copy_from_slice(clear);
        let mut cipher_packet = vec![0; 3 + clear.len() + 4];
        let mut scratch = vec![0; core::cmp::max(43, 16 + clear.len())];
        match critical_section::with(|cs| {
            self.0.borrow_ref_mut(cs).ccm.encrypt_packet(
                &mut ccm_data,
                &clear_packet,
                &mut cipher_packet,
                &mut scratch,
            )
        }) {
            Ok(()) => cipher.copy_from_slice(&cipher_packet[3 ..]),
            Err(_) => return Err(board::Error::World),
        }
        Ok(())
    }

    fn decrypt(
        &mut self, key: &[u8], iv: &[u8], cipher: &[u8], clear: &mut [u8],
    ) -> Result<(), board::Error> {
        let key = key.try_into().map_err(|_| board::Error::User)?;
        let iv = iv.try_into().map_err(|_| board::Error::User)?;
        if 251 < clear.len() || cipher.len() != clear.len() + 4 {
            return Err(board::Error::User);
        }
        let mut ccm_data = CcmData::new(key, iv);
        let mut cipher_packet = vec![0; 3 + clear.len() + 4];
        cipher_packet[1] = clear.len() as u8 + 4;
        cipher_packet[3 ..].copy_from_slice(cipher);
        let mut clear_packet = vec![0; 3 + clear.len()];
        let mut scratch = vec![0; core::cmp::max(43, 16 + clear.len() + 4)];
        match critical_section::with(|cs| {
            self.0.borrow_ref_mut(cs).ccm.decrypt_packet(
                &mut ccm_data,
                &mut clear_packet,
                &cipher_packet,
                &mut scratch,
            )
        }) {
            Ok(()) => clear.copy_from_slice(&clear_packet[3 ..]),
            Err(_) => return Err(board::Error::World),
        }
        Ok(())
    }
}
