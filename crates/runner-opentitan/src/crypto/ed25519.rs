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

// sw/device/lib/crypto/include/ed25519.h

use wasefire_error::Error;

use crate::crypto::common::{
    BlindedKey, ConstByteBuf, ConstWord32Buf, EccKeyMode, KeyMode, OwnedBlindedKey,
    OwnedUnblindedKey, UnblindedKey, Word32Buf,
};
use crate::error::unwrap_status;
use crate::hardened_bool;

// otcrypto_eddsa_sign_mode_t
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SignMode {
    Eddsa = 0xae1,
}

impl SignMode {
    pub fn to_c(self) -> i32 {
        self as i32
    }
}

pub fn keygen() -> Result<(OwnedBlindedKey, OwnedUnblindedKey), Error> {
    let mut private = OwnedBlindedKey::new(KeyMode::Ecc(EccKeyMode::Ed25519))?;
    let mut public = OwnedUnblindedKey::new(KeyMode::Ecc(EccKeyMode::Ed25519))?;
    unwrap_status(unsafe { otcrypto_ed25519_keygen(&mut private.0, &mut public.0) })?;
    Ok((private, public))
}

pub fn sign(private: &BlindedKey, message: &[u8], signature: &mut [u32; 16]) -> Result<(), Error> {
    unwrap_status(unsafe {
        otcrypto_ed25519_sign(
            private,
            message.into(),
            SignMode::Eddsa.to_c(),
            signature.as_mut_slice().into(),
        )
    })?;
    Ok(())
}

pub fn verify(public: &UnblindedKey, message: &[u8], signature: &[u32; 16]) -> Result<bool, Error> {
    let mut result = 0i32;
    unwrap_status(unsafe {
        otcrypto_ed25519_verify(
            public,
            message.into(),
            SignMode::Eddsa.to_c(),
            signature.as_slice().into(),
            &mut result,
        )
    })?;
    Ok(result == hardened_bool::TRUE)
}

unsafe extern "C" {
    fn otcrypto_ed25519_keygen(private: *mut BlindedKey, public: *mut UnblindedKey) -> i32;
    fn otcrypto_ed25519_sign(
        private: *const BlindedKey, message: ConstByteBuf, mode: i32, signature: Word32Buf,
    ) -> i32;
    fn otcrypto_ed25519_verify(
        public: *const UnblindedKey, message: ConstByteBuf, mode: i32, signature: ConstWord32Buf,
        result: *mut i32,
    ) -> i32;
}
