// Copyright 2023 Google LLC
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

//! Provides AES-256-GCM.

use alloc::vec;
use alloc::vec::Vec;

#[cfg(feature = "rust-crypto")]
pub use rust_crypto::*;
use wasefire_applet_api::crypto::gcm as api;

use super::Error;

/// Describes AES-256-GCM support.
pub struct Support {
    /// The [`encrypt`] and [`decrypt`] functions are supported without copy when the input pointer
    /// is non-null, i.e. the function uses different buffers for input and output.
    pub no_copy: bool,

    /// The [`encrypt`] and [`decrypt`] functions are supported without copy when the input pointer
    /// is null, i.e. the function operates in-place in the same buffer.
    pub in_place_no_copy: bool,
}

pub struct Cipher {
    pub text: Vec<u8>,
    pub tag: [u8; 16],
}

/// Whether AES-256-GCM is supported.
pub fn is_supported() -> bool {
    let api::support::Results { support } = unsafe { api::support() };
    support != 0
}

/// Describes how AES-256-GCM is supported.
pub fn support() -> Support {
    let api::support::Results { support } = unsafe { api::support() };
    Support {
        no_copy: (support & 1 << api::Support::NoCopy as u32) != 0,
        in_place_no_copy: (support & 1 << api::Support::InPlaceNoCopy as u32) != 0,
    }
}

/// Encrypts and authenticates a cleartext.
pub fn encrypt(key: &[u8; 32], iv: &[u8; 12], aad: &[u8], clear: &[u8]) -> Result<Cipher, Error> {
    let mut text = vec![0; clear.len()];
    let tag = encrypt_mut(key, iv, aad, clear, &mut text)?;
    Ok(Cipher { text, tag })
}

/// Encrypts and authenticates a cleartext to a ciphertext.
pub fn encrypt_mut(
    key: &[u8; 32], iv: &[u8; 12], aad: &[u8], clear: &[u8], cipher: &mut [u8],
) -> Result<[u8; 16], Error> {
    if clear.len() != cipher.len() {
        return Err(Error::InvalidArgument);
    }
    let mut tag = [0; 16];
    let params = api::encrypt::Params {
        key: key.as_ptr(),
        iv: iv.as_ptr(),
        aad: aad.as_ptr(),
        aad_len: aad.len(),
        length: clear.len(),
        clear: clear.as_ptr(),
        cipher: cipher.as_mut_ptr(),
        tag: tag.as_mut_ptr(),
    };
    let api::encrypt::Results { res } = unsafe { api::encrypt(params) };
    Error::to_result(res)?;
    Ok(tag)
}

/// Encrypts and authenticates a buffer in place.
pub fn encrypt_in_place(
    key: &[u8; 32], iv: &[u8; 12], aad: &[u8], buffer: &mut [u8],
) -> Result<[u8; 16], Error> {
    let mut tag = [0; 16];
    let params = api::encrypt::Params {
        key: key.as_ptr(),
        iv: iv.as_ptr(),
        aad: aad.as_ptr(),
        aad_len: aad.len(),
        length: buffer.len(),
        clear: core::ptr::null(),
        cipher: buffer.as_mut_ptr(),
        tag: tag.as_mut_ptr(),
    };
    let api::encrypt::Results { res } = unsafe { api::encrypt(params) };
    Error::to_result(res)?;
    Ok(tag)
}

/// Decrypts and authenticates a ciphertext.
pub fn decrypt(
    key: &[u8; 32], iv: &[u8; 12], aad: &[u8], cipher: &Cipher,
) -> Result<Vec<u8>, Error> {
    let mut clear = vec![0; cipher.text.len()];
    decrypt_mut(key, iv, aad, &cipher.tag, &cipher.text, &mut clear)?;
    Ok(clear)
}

/// Decrypts and authenticates a ciphertext to a cleartext.
pub fn decrypt_mut(
    key: &[u8; 32], iv: &[u8; 12], aad: &[u8], tag: &[u8; 16], cipher: &[u8], clear: &mut [u8],
) -> Result<(), Error> {
    if cipher.len() != clear.len() {
        return Err(Error::InvalidArgument);
    }
    let params = api::decrypt::Params {
        key: key.as_ptr(),
        iv: iv.as_ptr(),
        aad: aad.as_ptr(),
        aad_len: aad.len(),
        tag: tag.as_ptr(),
        length: cipher.len(),
        cipher: cipher.as_ptr(),
        clear: clear.as_mut_ptr(),
    };
    let api::decrypt::Results { res } = unsafe { api::decrypt(params) };
    Error::to_result(res)?;
    Ok(())
}

/// Decrypts and authenticates a ciphertext.
pub fn decrypt_in_place(
    key: &[u8; 32], iv: &[u8; 12], aad: &[u8], tag: &[u8; 16], buffer: &mut [u8],
) -> Result<(), Error> {
    let params = api::decrypt::Params {
        key: key.as_ptr(),
        iv: iv.as_ptr(),
        aad: aad.as_ptr(),
        aad_len: aad.len(),
        tag: tag.as_ptr(),
        length: buffer.len(),
        cipher: core::ptr::null(),
        clear: buffer.as_mut_ptr(),
    };
    let api::decrypt::Results { res } = unsafe { api::decrypt(params) };
    Error::to_result(res)?;
    Ok(())
}

#[cfg(feature = "rust-crypto")]
mod rust_crypto {
    use super::*;

    /// AES-256-GCM key parametric over in-place flavor.
    ///
    /// Prefer using [`Aes256Gcm`] or [`Aes256GcmInPlace`] instead.
    pub struct Key<const IN_PLACE: bool> {
        key: [u8; 32],
    }

    /// AES-256-GCM key to be used with the `Aead` trait.
    pub type Aes256Gcm = Key<false>;

    /// AES-256-GCM key to be used with the `AeadInPlace` trait.
    pub type Aes256GcmInPlace = Key<true>;

    impl<const IN_PLACE: bool> aead::KeySizeUser for Key<IN_PLACE> {
        type KeySize = aead::consts::U32;
    }

    impl<const IN_PLACE: bool> aead::KeyInit for Key<IN_PLACE> {
        fn new(key: &aead::Key<Self>) -> Self {
            Self { key: (*key).into() }
        }
    }

    impl<const IN_PLACE: bool> aead::AeadCore for Key<IN_PLACE> {
        type NonceSize = aead::consts::U12;
        type TagSize = aead::consts::U16;
        type CiphertextOverhead = aead::consts::U0;
    }

    impl aead::Aead for Key<false> {
        fn encrypt<'msg, 'aad>(
            &self, nonce: &aead::Nonce<Self>, plaintext: impl Into<aead::Payload<'msg, 'aad>>,
        ) -> aead::Result<Vec<u8>> {
            let payload = plaintext.into();
            let len = payload.msg.len();
            let mut result = vec![0; len + 16];
            let tag = encrypt_mut(
                &self.key,
                nonce.as_ref(),
                payload.aad,
                payload.msg,
                &mut result[.. len],
            )
            .map_err(|_| aead::Error)?;
            result[len ..].copy_from_slice(tag.as_ref());
            Ok(result)
        }

        fn decrypt<'msg, 'aad>(
            &self, nonce: &aead::Nonce<Self>, ciphertext: impl Into<aead::Payload<'msg, 'aad>>,
        ) -> aead::Result<Vec<u8>> {
            let payload: aead::Payload = ciphertext.into();
            let len = payload.msg.len().checked_sub(16).ok_or(aead::Error)?;
            let (cipher, tag) = payload.msg.split_at(len);
            let mut clear = vec![0; len];
            decrypt_mut(
                &self.key,
                nonce.as_ref(),
                payload.aad,
                tag.try_into().unwrap(),
                cipher,
                &mut clear,
            )
            .map_err(|_| aead::Error)?;
            Ok(clear)
        }
    }

    impl aead::AeadInPlace for Key<true> {
        fn encrypt_in_place_detached(
            &self, nonce: &aead::Nonce<Self>, associated_data: &[u8], buffer: &mut [u8],
        ) -> aead::Result<aead::Tag<Self>> {
            encrypt_in_place(&self.key, nonce.as_ref(), associated_data, buffer)
                .map(|x| x.into())
                .map_err(|_| aead::Error)
        }

        fn decrypt_in_place_detached(
            &self, nonce: &aead::Nonce<Self>, associated_data: &[u8], buffer: &mut [u8],
            tag: &aead::Tag<Self>,
        ) -> aead::Result<()> {
            decrypt_in_place(&self.key, nonce.as_ref(), associated_data, tag.as_ref(), buffer)
                .map(|x| x.into())
                .map_err(|_| aead::Error)
        }
    }
}
