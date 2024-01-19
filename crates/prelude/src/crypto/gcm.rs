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

use crate::{convert, convert_unit, Error};

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
    pub tag: Vec<u8>,
}

/// Whether AES-256-GCM is supported.
pub fn is_supported() -> bool {
    convert(unsafe { api::support() }).unwrap_or(0) != 0
}

/// Describes how AES-256-GCM is supported.
pub fn support() -> Support {
    let support = convert(unsafe { api::support() }).unwrap();
    Support {
        no_copy: (support & 1 << api::Support::NoCopy as u32) != 0,
        in_place_no_copy: (support & 1 << api::Support::InPlaceNoCopy as u32) != 0,
    }
}

/// Returns the supported tag length.
pub fn tag_length() -> usize {
    convert(unsafe { api::tag_length() }).unwrap()
}

/// Encrypts and authenticates a cleartext.
pub fn encrypt(key: &[u8; 32], iv: &[u8; 12], aad: &[u8], clear: &[u8]) -> Result<Cipher, Error> {
    let mut text = vec![0; clear.len()];
    let mut tag = vec![0; tag_length()];
    encrypt_mut(key, iv, aad, clear, &mut text, &mut tag)?;
    Ok(Cipher { text, tag })
}

/// Encrypts and authenticates a cleartext to a ciphertext.
pub fn encrypt_mut(
    key: &[u8; 32], iv: &[u8; 12], aad: &[u8], clear: &[u8], cipher: &mut [u8], tag: &mut [u8],
) -> Result<(), Error> {
    if clear.len() != cipher.len() || tag.len() != tag_length() {
        return Err(Error::user(0));
    }
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
    convert_unit(unsafe { api::encrypt(params) })
}

/// Encrypts and authenticates a buffer in place.
pub fn encrypt_in_place(
    key: &[u8; 32], iv: &[u8; 12], aad: &[u8], buffer: &mut [u8], tag: &mut [u8],
) -> Result<(), Error> {
    if tag.len() != tag_length() {
        return Err(Error::user(0));
    }
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
    convert_unit(unsafe { api::encrypt(params) })
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
    key: &[u8; 32], iv: &[u8; 12], aad: &[u8], tag: &[u8], cipher: &[u8], clear: &mut [u8],
) -> Result<(), Error> {
    if cipher.len() != clear.len() || tag.len() != tag_length() {
        return Err(Error::user(0));
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
    convert_unit(unsafe { api::decrypt(params) })
}

/// Decrypts and authenticates a ciphertext.
pub fn decrypt_in_place(
    key: &[u8; 32], iv: &[u8; 12], aad: &[u8], tag: &[u8], buffer: &mut [u8],
) -> Result<(), Error> {
    if tag.len() != tag_length() {
        return Err(Error::user(0));
    }
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
    convert_unit(unsafe { api::decrypt(params) })
}

#[cfg(feature = "rust-crypto")]
mod rust_crypto {
    use super::*;

    /// AES-256-GCM key parametric over in-place flavor.
    ///
    /// Prefer using [`Aes256Gcm`] or [`Aes256GcmInPlace`] instead.
    #[derive(zeroize::Zeroize, zeroize::ZeroizeOnDrop)]
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
        // This is the maximum tag size. We can't know at compile-time the actual supported tag
        // length. This means we pad with zeros the tag. The user must truncate the tag.
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
            let mut tag = [0; 16];
            encrypt_mut(
                &self.key,
                nonce.as_ref(),
                payload.aad,
                payload.msg,
                &mut result[.. len],
                &mut tag[.. tag_length()],
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
                &tag[.. tag_length()],
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
            let mut tag = [0; 16];
            encrypt_in_place(
                &self.key,
                nonce.as_ref(),
                associated_data,
                buffer,
                &mut tag[.. tag_length()],
            )
            .map_err(|_| aead::Error)?;
            Ok(tag.into())
        }

        fn decrypt_in_place_detached(
            &self, nonce: &aead::Nonce<Self>, associated_data: &[u8], buffer: &mut [u8],
            tag: &aead::Tag<Self>,
        ) -> aead::Result<()> {
            decrypt_in_place(
                &self.key,
                nonce.as_ref(),
                associated_data,
                &tag[.. tag_length()],
                buffer,
            )
            .map_err(|_| aead::Error)
        }
    }
}
