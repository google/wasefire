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

//! AES-256-GCM.

use crate::{Error, Unimplemented, Unsupported};

/// AES-256-GCM interface.
pub trait Api {
    /// Whether AES-256-GCM is supported.
    fn is_supported(&mut self) -> bool;

    /// Encrypts and authenticates a clear text with associated data given a key and IV.
    ///
    /// The clear- and cipher-texts must have the same length. If the clear text is omitted, then
    /// the cipher text is encrypted in place.
    fn encrypt(
        &mut self, key: &[u8; 32], iv: &[u8; 12], aad: &[u8], clear: Option<&[u8]>,
        cipher: &mut [u8], tag: &mut [u8; 16],
    ) -> Result<(), Error>;

    /// Decrypts and authenticates a cipher text with associated data given a key and IV.
    ///
    /// The cipher- and clear-texts must have the same length. If the cipher text is omitted, then
    /// the clear text is decrypted in place.
    fn decrypt(
        &mut self, key: &[u8; 32], iv: &[u8; 12], aad: &[u8], tag: &[u8; 16],
        cipher: Option<&[u8]>, clear: &mut [u8],
    ) -> Result<(), Error>;
}

impl Api for Unimplemented {
    fn is_supported(&mut self) -> bool {
        unreachable!()
    }

    fn encrypt(
        &mut self, _: &[u8; 32], _: &[u8; 12], _: &[u8], _: Option<&[u8]>, _: &mut [u8],
        _: &mut [u8; 16],
    ) -> Result<(), Error> {
        unreachable!()
    }

    fn decrypt(
        &mut self, _: &[u8; 32], _: &[u8; 12], _: &[u8], _: &[u8; 16], _: Option<&[u8]>,
        _: &mut [u8],
    ) -> Result<(), Error> {
        unreachable!()
    }
}

#[cfg(not(feature = "software-crypto-aes256-gcm"))]
mod unsupported {
    use super::*;

    impl Api for Unsupported {
        fn is_supported(&mut self) -> bool {
            false
        }

        fn encrypt(
            &mut self, _: &[u8; 32], _: &[u8; 12], _: &[u8], _: Option<&[u8]>, _: &mut [u8],
            _: &mut [u8; 16],
        ) -> Result<(), Error> {
            Err(Error::User)
        }

        fn decrypt(
            &mut self, _: &[u8; 32], _: &[u8; 12], _: &[u8], _: &[u8; 16], _: Option<&[u8]>,
            _: &mut [u8],
        ) -> Result<(), Error> {
            Err(Error::User)
        }
    }
}

#[cfg(feature = "software-crypto-aes256-gcm")]
mod unsupported {
    use aes_gcm::{AeadInPlace, Aes256Gcm, KeyInit};

    use super::*;

    impl Api for Unsupported {
        fn is_supported(&mut self) -> bool {
            true
        }

        fn encrypt(
            &mut self, key: &[u8; 32], iv: &[u8; 12], aad: &[u8], clear: Option<&[u8]>,
            cipher: &mut [u8], tag: &mut [u8; 16],
        ) -> Result<(), Error> {
            let gcm = Aes256Gcm::new(key.into());
            if let Some(clear) = clear {
                cipher.copy_from_slice(clear);
            }
            tag.copy_from_slice(
                &gcm.encrypt_in_place_detached(iv.into(), aad, cipher).map_err(|_| Error::World)?,
            );
            Ok(())
        }

        fn decrypt(
            &mut self, key: &[u8; 32], iv: &[u8; 12], aad: &[u8], tag: &[u8; 16],
            cipher: Option<&[u8]>, clear: &mut [u8],
        ) -> Result<(), Error> {
            let gcm = Aes256Gcm::new(key.into());
            if let Some(cipher) = cipher {
                clear.copy_from_slice(cipher);
            }
            gcm.decrypt_in_place_detached(iv.into(), aad, clear, tag.into())
                .map_err(|_| Error::World)
        }
    }
}
