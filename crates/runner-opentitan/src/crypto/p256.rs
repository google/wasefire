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

// sw/device/lib/crypto/include/aes.h

use wasefire_error::Error;

use crate::crypto::common::{
    BlindedKey, ConstWord32Buf, HashDigest, HashMode, KeyMode, OwnedBlindedKey, OwnedUnblindedKey,
    UnblindedKey, Word32Buf,
};
use crate::error::unwrap_status;
use crate::hardened_bool;

pub fn ecdsa_keygen() -> Result<(OwnedBlindedKey, OwnedUnblindedKey), Error> {
    let mut private = OwnedBlindedKey::new(KeyMode::EcdsaP256)?;
    let mut public = OwnedUnblindedKey::new(KeyMode::EcdsaP256)?;
    unwrap_status(unsafe { otcrypto_ecdsa_p256_keygen(&mut private.0, &mut public.0) })?;
    Ok((private, public))
}

pub fn ecdsa_sign(
    private: &BlindedKey, digest: &[u32], signature: &mut [u32],
) -> Result<(), Error> {
    let digest = HashDigest {
        mode: HashMode::Sha256.to_c(),
        data: digest.as_ptr() as *mut u32,
        len: digest.len(),
    };
    unwrap_status(unsafe { otcrypto_ecdsa_p256_sign(private, digest, signature.into()) })?;
    Ok(())
}

pub fn ecdsa_verify(
    public: &UnblindedKey, digest: &[u32], signature: &[u32],
) -> Result<bool, Error> {
    let digest = HashDigest {
        mode: HashMode::Sha256.to_c(),
        data: digest.as_ptr() as *mut u32,
        len: digest.len(),
    };
    let mut result = 0i32;
    unwrap_status(unsafe {
        otcrypto_ecdsa_p256_verify(public, digest, signature.into(), &mut result)
    })?;
    Ok(result == hardened_bool::TRUE)
}

pub fn ecdh_keygen() -> Result<(OwnedBlindedKey, OwnedUnblindedKey), Error> {
    let mut private = OwnedBlindedKey::new(KeyMode::EcdhP256)?;
    let mut public = OwnedUnblindedKey::new(KeyMode::EcdhP256)?;
    unwrap_status(unsafe { otcrypto_ecdh_p256_keygen(&mut private.0, &mut public.0) })?;
    Ok((private, public))
}

pub fn ecdh(
    private: &BlindedKey, public: &UnblindedKey, key_mode: KeyMode,
) -> Result<OwnedBlindedKey, Error> {
    let mut shared = OwnedBlindedKey::new(key_mode)?;
    unwrap_status(unsafe { otcrypto_ecdh_p256(private, public, &mut shared.0) })?;
    Ok(shared)
}

unsafe extern "C" {
    fn otcrypto_ecdsa_p256_keygen(private: *mut BlindedKey, public: *mut UnblindedKey) -> i32;
    fn otcrypto_ecdsa_p256_sign(
        private: *const BlindedKey, digest: HashDigest, signature: Word32Buf,
    ) -> i32;
    fn otcrypto_ecdsa_p256_verify(
        public: *const UnblindedKey, digest: HashDigest, signature: ConstWord32Buf,
        result: *mut i32,
    ) -> i32;
    fn otcrypto_ecdh_p256_keygen(private: *mut BlindedKey, public: *mut UnblindedKey) -> i32;
    fn otcrypto_ecdh_p256(
        private: *const BlindedKey, public: *const UnblindedKey, shared: *mut BlindedKey,
    ) -> i32;
}
