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

use crypto_common::array::ArraySize;

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

/// AEAD interface.
pub trait Api<Key, Iv>: Support<AeadSupport> + Send
where
    Key: ArraySize,
    Iv: ArraySize,
{
    /// The tag length.
    type Tag: ArraySize;

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
pub type Array<N> = crypto_common::array::Array<u8, N>;

#[cfg(feature = "internal-software-crypto-aead")]
mod software {
    use aead::inout::InOutBuf;
    use aead::{AeadCore, AeadInOut};
    use crypto_common::{KeyInit, KeySizeUser};

    use super::*;

    impl<T: AeadInOut> Support<AeadSupport> for T {
        const SUPPORT: AeadSupport = AeadSupport { no_copy: false, in_place_no_copy: true };
    }

    impl<Key, Iv, T> Api<Key, Iv> for T
    where
        T: Send + KeyInit + AeadInOut,
        T: KeySizeUser<KeySize = Key>,
        T: AeadCore<NonceSize = Iv>,
        Key: ArraySize,
        Iv: ArraySize,
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
            let buffer = InOutBuf::from(cipher);
            tag.copy_from_slice(
                &aead.encrypt_inout_detached(iv, aad, buffer).map_err(|_| Error::world(0))?,
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
            let buffer = InOutBuf::from(clear);
            aead.decrypt_inout_detached(iv, aad, buffer, tag).map_err(|_| Error::world(0))
        }
    }
}
