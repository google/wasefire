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

//! Ed25519.

use core::alloc::Layout;

#[cfg(feature = "software-crypto-ed25519")]
pub use software::*;
use wasefire_error::Error;

use crate::Support;

/// Ed25519 interface.
pub trait Api: Support<bool> + Send {
    /// Layout of a private key.
    const PRIVATE: Layout;

    /// Layout of a public key.
    const PUBLIC: Layout;

    /// Length in bytes of a wrapped private key.
    const WRAPPED: usize;

    /// Generates a private key.
    fn generate(private: &mut [u8]) -> Result<(), Error>;

    /// Returns the public key of a private key.
    fn public(private: &[u8], public: &mut [u8]) -> Result<(), Error>;

    /// Signs a digest.
    fn sign(
        private: &[u8], message: &[u8], r: &mut [u8; 32], s: &mut [u8; 32],
    ) -> Result<(), Error>;

    /// Verifies a signature.
    fn verify(public: &[u8], message: &[u8], r: &[u8; 32], s: &[u8; 32]) -> Result<bool, Error>;

    /// Drops a private key.
    ///
    /// This may zeroize the storage for example. Implementations should not rely on this for memory
    /// leaks. This is only security relevant.
    fn drop_private(private: &mut [u8]) -> Result<(), Error>;

    /// Exports a private key.
    ///
    /// The wrapped key must be `Self::WRAPPED` bytes long. Users cannot assume that the wrapped key
    /// is the scalar representing the private key.
    fn export_private(private: &[u8], wrapped: &mut [u8]) -> Result<(), Error>;

    /// Imports a private key.
    ///
    /// The wrapped key must be `Self::WRAPPED` bytes long.
    fn import_private(wrapped: &[u8], private: &mut [u8]) -> Result<(), Error>;

    /// Exports a public key.
    fn export_public(public: &[u8], a: &mut [u8; 32]) -> Result<(), Error>;

    /// Imports a public key.
    fn import_public(a: &[u8; 32], public: &mut [u8]) -> Result<(), Error>;
}

#[cfg(feature = "software-crypto-ed25519")]
mod software {
    use core::marker::PhantomData;

    use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
    use signature::rand_core::CryptoRngCore;
    use wasefire_error::Code;
    use zeroize::Zeroize;

    use super::*;
    use crate::Supported;
    use crate::crypto::WithError;

    /// Ed25519 software implementation.
    pub struct Software<R> {
        rng: PhantomData<R>,
    }

    impl<R> Supported for Software<R> {}

    impl<R: Default + CryptoRngCore + WithError + Send> Api for Software<R> {
        const PRIVATE: Layout = unsafe { Layout::from_size_align_unchecked(32, 1) };
        const PUBLIC: Layout = unsafe { Layout::from_size_align_unchecked(32, 1) };
        const WRAPPED: usize = 32;

        fn generate(private: &mut [u8]) -> Result<(), Error> {
            if private.len() != 32 {
                return Err(Error::user(Code::InvalidLength));
            }
            let mut rng = R::default();
            let key = R::with_error(|| SigningKey::generate(&mut rng))?;
            private.copy_from_slice(&key.to_bytes());
            Ok(())
        }

        fn public(private: &[u8], public: &mut [u8]) -> Result<(), Error> {
            if private.len() != 32 || public.len() != 32 {
                return Err(Error::user(Code::InvalidLength));
            }
            let key = SigningKey::from_bytes(private.try_into().unwrap());
            public.copy_from_slice(key.verifying_key().as_bytes());
            Ok(())
        }

        fn sign(
            private: &[u8], message: &[u8], r: &mut [u8; 32], s: &mut [u8; 32],
        ) -> Result<(), Error> {
            if private.len() != 32 {
                return Err(Error::user(Code::InvalidLength));
            }
            let key = SigningKey::from_bytes(private.try_into().unwrap());
            let sig = key.try_sign(message).map_err(|_| Error::world(0))?;
            r.copy_from_slice(sig.r_bytes());
            s.copy_from_slice(sig.s_bytes());
            Ok(())
        }

        fn verify(
            public: &[u8], message: &[u8], r: &[u8; 32], s: &[u8; 32],
        ) -> Result<bool, Error> {
            if public.len() != 32 {
                return Err(Error::user(Code::InvalidLength));
            }
            let key =
                VerifyingKey::from_bytes(public.try_into().unwrap()).map_err(|_| Error::user(0))?;
            let sig = Signature::from_components(*r, *s);
            Ok(key.verify_strict(message, &sig).is_ok())
        }

        fn drop_private(private: &mut [u8]) -> Result<(), Error> {
            if private.len() != 32 {
                return Err(Error::user(Code::InvalidLength));
            }
            private.zeroize();
            Ok(())
        }

        fn export_private(private: &[u8], wrapped: &mut [u8]) -> Result<(), Error> {
            if private.len() != 32 || wrapped.len() != 32 {
                return Err(Error::user(Code::InvalidLength));
            }
            wrapped.copy_from_slice(private);
            Ok(())
        }

        fn import_private(wrapped: &[u8], private: &mut [u8]) -> Result<(), Error> {
            if wrapped.len() != 32 || private.len() != 32 {
                return Err(Error::user(Code::InvalidLength));
            }
            private.copy_from_slice(wrapped);
            Ok(())
        }

        fn export_public(public: &[u8], a: &mut [u8; 32]) -> Result<(), Error> {
            if public.len() != 32 {
                return Err(Error::user(Code::InvalidLength));
            }
            a.copy_from_slice(public);
            Ok(())
        }

        fn import_public(a: &[u8; 32], public: &mut [u8]) -> Result<(), Error> {
            if public.len() != 32 {
                return Err(Error::user(Code::InvalidLength));
            }
            public.copy_from_slice(a);
            Ok(())
        }
    }
}
