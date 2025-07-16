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

//! ECDSA.

use core::alloc::Layout;

#[cfg(feature = "internal-software-crypto-ecdsa")]
pub use software::*;
use wasefire_error::Error;

use crate::Support;

/// ECDSA interface.
pub trait Api<const N: usize>: Support<bool> + Send {
    /// Layout of a private key.
    ///
    /// This may use more than `N` bytes in case blinding or checksum are used. This may also
    /// require stricter alignment than one byte.
    const PRIVATE: Layout;

    /// Layout of a public key.
    ///
    /// This may be bigger than `2 * N` in case checksum is used. This may also require stricter
    /// alignment than one byte.
    const PUBLIC: Layout;

    /// Length in bytes of a wrapped private key.
    ///
    /// This may be greater than `N` in case blinding is used.
    const WRAPPED: usize;

    /// Generates a private key.
    fn generate(private: &mut [u8]) -> Result<(), Error>;

    /// Returns the public key of a private key.
    fn public(private: &[u8], public: &mut [u8]) -> Result<(), Error>;

    /// Signs a digest.
    ///
    /// The signature components `r` and `s` are in big-endian.
    fn sign(
        private: &[u8], digest: &[u8; N], r: &mut [u8; N], s: &mut [u8; N],
    ) -> Result<(), Error>;

    /// Verifies a signature.
    ///
    /// The signature components `r` and `s` are in big-endian.
    fn verify(public: &[u8], digest: &[u8; N], r: &[u8; N], s: &[u8; N]) -> Result<bool, Error>;

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
    ///
    /// The coordinates `x` and `y` are in big-endian.
    fn export_public(public: &[u8], x: &mut [u8; N], y: &mut [u8; N]) -> Result<(), Error>;

    /// Imports a public key.
    ///
    /// The coordinates `x` and `y` are in big-endian.
    fn import_public(x: &[u8; N], y: &[u8; N], public: &mut [u8]) -> Result<(), Error>;
}

#[cfg(feature = "internal-software-crypto-ecdsa")]
mod software {
    use core::marker::PhantomData;

    use crypto_common::BlockSizeUser;
    use crypto_common::generic_array::ArrayLength;
    use ecdsa::hazmat::{SignPrimitive, VerifyPrimitive};
    use ecdsa::{EncodedPoint, PrimeCurve, Signature, SignatureSize, SigningKey, VerifyingKey};
    use elliptic_curve::sec1::{FromEncodedPoint, ModulusSize, ToEncodedPoint};
    use elliptic_curve::zeroize::Zeroize;
    use elliptic_curve::{AffinePoint, CurveArithmetic, FieldBytes, FieldBytesSize, Scalar};
    use rand_core::CryptoRngCore;
    use signature::digest::{Digest, FixedOutput, FixedOutputReset};
    use signature::hazmat::PrehashVerifier;
    use wasefire_error::Code;

    use super::*;
    use crate::Supported;
    use crate::crypto::WithError;

    /// ECDSA software implementation.
    pub struct Software<C, D, R, const N: usize> {
        curve: PhantomData<C>,
        digest: PhantomData<D>,
        rng: PhantomData<R>,
    }

    impl<C, D, R, const N: usize> Supported for Software<C, D, R, N> {}

    impl<C, D, R, const N: usize> Api<N> for Software<C, D, R, N>
    where
        C: PrimeCurve + CurveArithmetic,
        Scalar<C>: SignPrimitive<C>,
        SignatureSize<C>: ArrayLength<u8>,
        AffinePoint<C>: FromEncodedPoint<C> + ToEncodedPoint<C> + VerifyPrimitive<C>,
        FieldBytesSize<C>: ModulusSize,
        D: Support<bool> + Send,
        D: Digest + BlockSizeUser + FixedOutput<OutputSize = FieldBytesSize<C>> + FixedOutputReset,
        R: Default + CryptoRngCore + WithError + Send,
    {
        const PRIVATE: Layout = unsafe { Layout::from_size_align_unchecked(N, 1) };
        const PUBLIC: Layout = unsafe { Layout::from_size_align_unchecked(2 * N, 1) };
        const WRAPPED: usize = N;

        fn generate(private: &mut [u8]) -> Result<(), Error> {
            if private.len() != N {
                return Err(Error::user(Code::InvalidLength));
            }
            let mut rng = R::default();
            let key = R::with_error(|| SigningKey::<C>::random(&mut rng))?;
            private.copy_from_slice(&key.to_bytes());
            Ok(())
        }

        fn public(private: &[u8], public: &mut [u8]) -> Result<(), Error> {
            if public.len() != 2 * N || private.len() != N {
                return Err(Error::user(Code::InvalidLength));
            }
            let key = SigningKey::<C>::from_bytes(FieldBytes::<C>::from_slice(private))
                .map_err(|_| Error::user(Code::InvalidArgument))?;
            let key = key.verifying_key().as_affine().to_encoded_point(false);
            let (x, y) = public.split_at_mut(N);
            x.copy_from_slice(key.x().ok_or(Error::user(0))?);
            y.copy_from_slice(key.y().ok_or(Error::user(0))?);
            Ok(())
        }

        fn sign(
            private: &[u8], digest: &[u8; N], r: &mut [u8; N], s: &mut [u8; N],
        ) -> Result<(), Error> {
            if private.len() != N {
                return Err(Error::user(Code::InvalidLength));
            }
            let key = SigningKey::<C>::from_bytes(FieldBytes::<C>::from_slice(private))
                .map_err(|_| Error::user(Code::InvalidArgument))?;
            let (sig, _) = key
                .as_nonzero_scalar()
                .try_sign_prehashed_rfc6979::<D>(FieldBytes::<C>::from_slice(digest), &[])
                .map_err(|_| Error::world(0))?;
            r.copy_from_slice(FieldBytes::<C>::from(sig.r()).as_slice());
            s.copy_from_slice(FieldBytes::<C>::from(sig.s()).as_slice());
            Ok(())
        }

        fn verify(
            public: &[u8], digest: &[u8; N], r: &[u8; N], s: &[u8; N],
        ) -> Result<bool, Error> {
            if public.len() != 2 * N {
                return Err(Error::user(Code::InvalidLength));
            }
            let (x, y) = public.split_at(N);
            let key = EncodedPoint::<C>::from_affine_coordinates(x.into(), y.into(), false);
            let key = VerifyingKey::<C>::from_encoded_point(&key).map_err(|_| Error::user(0))?;
            let r = FieldBytes::<C>::from_slice(r).clone();
            let s = FieldBytes::<C>::from_slice(s).clone();
            let sig = Signature::from_scalars(r, s).map_err(|_| Error::user(0))?;
            Ok(key.verify_prehash(digest, &sig).is_ok())
        }

        fn drop_private(private: &mut [u8]) -> Result<(), Error> {
            if private.len() != N {
                return Err(Error::user(Code::InvalidLength));
            }
            private.zeroize();
            Ok(())
        }

        fn export_private(private: &[u8], wrapped: &mut [u8]) -> Result<(), Error> {
            if private.len() != N || wrapped.len() != N {
                return Err(Error::user(Code::InvalidLength));
            }
            wrapped.copy_from_slice(private);
            Ok(())
        }

        fn import_private(wrapped: &[u8], private: &mut [u8]) -> Result<(), Error> {
            if wrapped.len() != N || private.len() != N {
                return Err(Error::user(Code::InvalidLength));
            }
            private.copy_from_slice(wrapped);
            Ok(())
        }

        fn export_public(public: &[u8], x: &mut [u8; N], y: &mut [u8; N]) -> Result<(), Error> {
            if public.len() != 2 * N {
                return Err(Error::user(Code::InvalidLength));
            }
            let (pub_x, pub_y) = public.split_at(N);
            x.copy_from_slice(pub_x);
            y.copy_from_slice(pub_y);
            Ok(())
        }

        fn import_public(x: &[u8; N], y: &[u8; N], public: &mut [u8]) -> Result<(), Error> {
            if public.len() != 2 * N {
                return Err(Error::user(Code::InvalidLength));
            }
            let (pub_x, pub_y) = public.split_at_mut(N);
            pub_x.copy_from_slice(x);
            pub_y.copy_from_slice(y);
            Ok(())
        }
    }
}
