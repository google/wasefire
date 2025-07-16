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

//! ECDH.

use core::alloc::Layout;

#[cfg(feature = "internal-software-crypto-ecdh")]
pub use software::*;
use wasefire_error::Error;

use crate::Support;

/// ECDH interface.
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

    /// Layout of a shared secret.
    ///
    /// This may be bigger than `N` in case blinding or checksum or used. This may also require
    /// stricter alignment than one byte.
    const SHARED: Layout;

    /// Generates a private key.
    fn generate(private: &mut [u8]) -> Result<(), Error>;

    /// Returns the public key of a private key.
    fn public(private: &[u8], public: &mut [u8]) -> Result<(), Error>;

    /// Computes the shared secret of a private and public key.
    fn shared(private: &[u8], public: &[u8], shared: &mut [u8]) -> Result<(), Error>;

    /// Drops a private key.
    ///
    /// This may zeroize the storage for example. Implementations should not rely on this for memory
    /// leaks. This is only security relevant.
    fn drop_private(private: &mut [u8]) -> Result<(), Error>;

    /// Drops a shared secret.
    ///
    /// This may zeroize the storage for example. Implementations should not rely on this for memory
    /// leaks. This is only security relevant.
    fn drop_shared(shared: &mut [u8]) -> Result<(), Error>;

    /// Exports a public key.
    ///
    /// The coordinates `x` and `y` are in big-endian.
    fn export_public(public: &[u8], x: &mut [u8; N], y: &mut [u8; N]) -> Result<(), Error>;

    /// Imports a public key.
    ///
    /// The coordinates `x` and `y` are in big-endian.
    fn import_public(x: &[u8; N], y: &[u8; N], public: &mut [u8]) -> Result<(), Error>;

    /// Exports a shared secret.
    ///
    /// The coordinate `x` is in big-endian.
    fn export_shared(shared: &[u8], x: &mut [u8; N]) -> Result<(), Error>;
}

#[cfg(feature = "internal-software-crypto-ecdh")]
mod software {
    use core::marker::PhantomData;

    use elliptic_curve::sec1::{EncodedPoint, FromEncodedPoint, ModulusSize, ToEncodedPoint};
    use elliptic_curve::subtle::CtOption;
    use elliptic_curve::zeroize::Zeroize;
    use elliptic_curve::{
        AffinePoint, CurveArithmetic, FieldBytes, FieldBytesSize, ProjectivePoint, Scalar,
        ScalarPrimitive, SecretKey,
    };
    use rand_core::CryptoRngCore;
    use wasefire_error::Code;

    use super::*;
    use crate::Supported;
    use crate::crypto::WithError;

    /// ECDH software implementation.
    pub struct Software<C, R, const N: usize> {
        curve: PhantomData<C>,
        rng: PhantomData<R>,
    }

    impl<C, R, const N: usize> Supported for Software<C, R, N> {}

    impl<C, R, const N: usize> Api<N> for Software<C, R, N>
    where
        C: CurveArithmetic,
        AffinePoint<C>: FromEncodedPoint<C> + ToEncodedPoint<C>,
        ProjectivePoint<C>: FromEncodedPoint<C>,
        FieldBytesSize<C>: ModulusSize,
        R: Default + CryptoRngCore + WithError + Send,
    {
        const PRIVATE: Layout = unsafe { Layout::from_size_align_unchecked(N, 1) };
        const PUBLIC: Layout = unsafe { Layout::from_size_align_unchecked(2 * N, 1) };
        const SHARED: Layout = unsafe { Layout::from_size_align_unchecked(N, 1) };

        fn generate(private: &mut [u8]) -> Result<(), Error> {
            if private.len() != N {
                return Err(Error::user(Code::InvalidLength));
            }
            let mut rng = R::default();
            let key = R::with_error(|| SecretKey::<C>::random(&mut rng))?;
            private.copy_from_slice(&key.to_bytes());
            Ok(())
        }

        fn public(private: &[u8], public: &mut [u8]) -> Result<(), Error> {
            if private.len() != N || public.len() != 2 * N {
                return Err(Error::user(Code::InvalidLength));
            }
            let key = SecretKey::<C>::from_bytes(FieldBytes::<C>::from_slice(private))
                .map_err(|_| Error::user(Code::InvalidArgument))?;
            let key = key.public_key().as_affine().to_encoded_point(false);
            let (x, y) = public.split_at_mut(N);
            x.copy_from_slice(key.x().ok_or(Error::user(0))?);
            y.copy_from_slice(key.y().ok_or(Error::user(0))?);
            Ok(())
        }

        fn shared(private: &[u8], public: &[u8], shared: &mut [u8]) -> Result<(), Error> {
            if private.len() != N || public.len() != 2 * N || shared.len() != N {
                return Err(Error::user(Code::InvalidLength));
            }
            let private = unwrap_ct(ScalarPrimitive::<C>::from_bytes(private.into()))?;
            let private = Scalar::<C>::from(private);
            let (x, y) = public.split_at(N);
            let public = EncodedPoint::<C>::from_affine_coordinates(x.into(), y.into(), false);
            let public = unwrap_ct(ProjectivePoint::<C>::from_encoded_point(&public))?;
            let secret: AffinePoint<C> = (public * private).into();
            let secret = secret.to_encoded_point(false);
            shared.copy_from_slice(secret.x().ok_or(Error::user(0))?);
            Ok(())
        }

        fn drop_private(private: &mut [u8]) -> Result<(), Error> {
            if private.len() != N {
                return Err(Error::user(Code::InvalidLength));
            }
            private.zeroize();
            Ok(())
        }

        fn drop_shared(shared: &mut [u8]) -> Result<(), Error> {
            if shared.len() != N {
                return Err(Error::user(Code::InvalidLength));
            }
            shared.zeroize();
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

        fn export_shared(shared: &[u8], x: &mut [u8; N]) -> Result<(), Error> {
            if shared.len() != N {
                return Err(Error::user(Code::InvalidLength));
            }
            x.copy_from_slice(shared);
            Ok(())
        }
    }

    fn unwrap_ct<T>(x: CtOption<T>) -> Result<T, Error> {
        Option::<T>::from(x).ok_or(Error::user(0))
    }
}
