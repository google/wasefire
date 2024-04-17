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

//! Authenticated Encryption with Associated Data.

use generic_array::{ArrayLength, GenericArray};

use crate::{Error, Support};

/// Describes how AEAD is supported.
#[derive(Copy, Clone)]
pub struct AeadSupport {
    /// The implementation doesn't copy when the input and output are in distinct buffers.
    pub no_copy: bool,

    /// The implementation doesn't copy when the input and output are in the same buffer.
    pub in_place_no_copy: bool,
}

impl From<AeadSupport> for bool {
    fn from(value: AeadSupport) -> Self {
        value.no_copy || value.in_place_no_copy
    }
}

/// Elliptic-curve cryptography interface.
pub trait Api<Key, Iv>: Support<AeadSupport> + Send
where
    Key: ArrayLength<u8>,
    Iv: ArrayLength<u8>,
{
    /// The tag length.
    type Tag: ArrayLength<u8>;

    /// Encrypts and authenticates a clear text with associated data given a key and IV.
    ///
    /// The clear- and cipher-texts must have the same length. If the clear text is omitted, then
    /// the cipher text is encrypted in place.
    fn encrypt(
        key: &Array<Key>, iv: &Array<Iv>, aad: &[u8], clear: Option<&[u8]>, cipher: &mut [u8],
        tag: &mut Array<Self::Tag>,
    ) -> Result<(), Error>;

    /// Decrypts and authenticates a cipher text with associated data given a key and IV.
    ///
    /// The cipher- and clear-texts must have the same length. If the cipher text is omitted, then
    /// the clear text is decrypted in place.
    fn decrypt(
        key: &Array<Key>, iv: &Array<Iv>, aad: &[u8], cipher: Option<&[u8]>,
        tag: &Array<Self::Tag>, clear: &mut [u8],
    ) -> Result<(), Error>;
}

/// Sequence of N bytes.
pub type Array<N> = GenericArray<u8, N>;

#[cfg(feature = "internal-software-crypto-aead")]
mod software {
    use aead::{AeadCore, AeadInPlace};
    use crypto_common::{KeyInit, KeySizeUser};

    use super::*;

    impl<T: AeadInPlace> Support<AeadSupport> for T {
        const SUPPORT: AeadSupport = AeadSupport { no_copy: false, in_place_no_copy: true };
    }

    impl<Key, Iv, T> Api<Key, Iv> for T
    where
        T: Send + KeyInit + AeadInPlace,
        T: KeySizeUser<KeySize = Key>,
        T: AeadCore<NonceSize = Iv>,
        Key: ArrayLength<u8>,
        Iv: ArrayLength<u8>,
    {
        type Tag = T::TagSize;

        fn encrypt(
            key: &Array<Key>, iv: &Array<Iv>, aad: &[u8], clear: Option<&[u8]>, cipher: &mut [u8],
            tag: &mut Array<Self::Tag>,
        ) -> Result<(), Error> {
            let aead = T::new(key);
            if let Some(clear) = clear {
                cipher.copy_from_slice(clear);
            }
            tag.copy_from_slice(
                &aead.encrypt_in_place_detached(iv, aad, cipher).map_err(|_| Error::world(0))?,
            );
            Ok(())
        }

        fn decrypt(
            key: &Array<Key>, iv: &Array<Iv>, aad: &[u8], cipher: Option<&[u8]>,
            tag: &Array<Self::Tag>, clear: &mut [u8],
        ) -> Result<(), Error> {
            let aead = T::new(key);
            if let Some(cipher) = cipher {
                clear.copy_from_slice(cipher);
            }
            aead.decrypt_in_place_detached(iv, aad, clear, tag).map_err(|_| Error::world(0))
        }
    }
}
