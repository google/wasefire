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

//! Elliptic-curve cryptography.

use generic_array::{ArrayLength, GenericArray};
#[cfg(feature = "internal-software-crypto-ecc")]
pub use software::*;

use crate::{Error, Support, Unsupported};

/// Elliptic-curve cryptography interface.
pub trait Api<N: ArrayLength<u8>>: Support<bool> + Send {
    /// Returns whether a scalar is valid.
    fn is_valid_scalar(n: &Int<N>) -> bool;

    /// Returns whether a point is valid.
    fn is_valid_point(x: &Int<N>, y: &Int<N>) -> bool;

    /// Base point multiplication.
    fn base_point_mul(n: &Int<N>, x: &mut Int<N>, y: &mut Int<N>) -> Result<(), Error>;

    /// Point multiplication.
    fn point_mul(
        n: &Int<N>, in_x: &Int<N>, in_y: &Int<N>, out_x: &mut Int<N>, out_y: &mut Int<N>,
    ) -> Result<(), Error>;

    /// ECDSA signature.
    fn ecdsa_sign(d: &Int<N>, m: &Int<N>, r: &mut Int<N>, s: &mut Int<N>) -> Result<(), Error>;

    /// ECDSA verification.
    fn ecdsa_verify(
        m: &Int<N>, x: &Int<N>, y: &Int<N>, r: &Int<N>, s: &Int<N>,
    ) -> Result<bool, Error>;
}

/// SEC-1 encoding of an `N` bytes integer.
pub type Int<N> = GenericArray<u8, N>;

impl<N: ArrayLength<u8>> Api<N> for Unsupported {
    fn is_valid_scalar(_: &Int<N>) -> bool {
        unreachable!()
    }

    fn is_valid_point(_: &Int<N>, _: &Int<N>) -> bool {
        unreachable!()
    }

    fn base_point_mul(_: &Int<N>, _: &mut Int<N>, _: &mut Int<N>) -> Result<(), Error> {
        unreachable!()
    }

    fn point_mul(
        _: &Int<N>, _: &Int<N>, _: &Int<N>, _: &mut Int<N>, _: &mut Int<N>,
    ) -> Result<(), Error> {
        unreachable!()
    }

    fn ecdsa_sign(_: &Int<N>, _: &Int<N>, _: &mut Int<N>, _: &mut Int<N>) -> Result<(), Error> {
        unreachable!()
    }

    fn ecdsa_verify(
        _: &Int<N>, _: &Int<N>, _: &Int<N>, _: &Int<N>, _: &Int<N>,
    ) -> Result<bool, Error> {
        unreachable!()
    }
}

#[cfg(feature = "internal-software-crypto-ecc")]
mod software {
    use core::marker::PhantomData;

    use crypto_common::BlockSizeUser;
    use digest::{Digest, FixedOutput, FixedOutputReset};
    use ecdsa::hazmat::{SignPrimitive, VerifyPrimitive};
    use ecdsa::{PrimeCurve, Signature, SignatureSize, VerifyingKey};
    use elliptic_curve::ops::MulByGenerator;
    use elliptic_curve::sec1::{EncodedPoint, FromEncodedPoint, ModulusSize, ToEncodedPoint};
    use elliptic_curve::subtle::CtOption;
    use elliptic_curve::{
        AffinePoint, CurveArithmetic, FieldBytesSize, ProjectivePoint, Scalar, ScalarPrimitive,
    };
    use signature::hazmat::PrehashVerifier;

    use super::*;
    use crate::Support;

    pub struct Software<C, D> {
        curve: PhantomData<C>,
        digest: PhantomData<D>,
    }

    type Int<C> = super::Int<FieldBytesSize<C>>;

    impl<C, D: Support<bool>> Support<bool> for Software<C, D> {
        const SUPPORT: bool = D::SUPPORT;
    }

    impl<C, D> Api<FieldBytesSize<C>> for Software<C, D>
    where
        C: Send + PrimeCurve + CurveArithmetic,
        D: Support<bool> + Send,
        D: Digest + BlockSizeUser + FixedOutput<OutputSize = FieldBytesSize<C>> + FixedOutputReset,
        AffinePoint<C>: FromEncodedPoint<C> + ToEncodedPoint<C> + VerifyPrimitive<C>,
        ProjectivePoint<C>: FromEncodedPoint<C>,
        SignatureSize<C>: ArrayLength<u8>,
        Scalar<C>: SignPrimitive<C>,
        FieldBytesSize<C>: ModulusSize,
    {
        fn is_valid_scalar(n: &Int<C>) -> bool {
            Self::scalar_from_int(n).is_ok()
        }

        fn is_valid_point(x: &Int<C>, y: &Int<C>) -> bool {
            Self::point_from_ints(x, y).is_ok()
        }

        fn base_point_mul(n: &Int<C>, x: &mut Int<C>, y: &mut Int<C>) -> Result<(), Error> {
            let r = ProjectivePoint::<C>::mul_by_generator(&Self::scalar_from_int(n)?);
            Self::point_to_ints(&r.into(), x, y)
        }

        fn point_mul(
            n: &Int<C>, in_x: &Int<C>, in_y: &Int<C>, out_x: &mut Int<C>, out_y: &mut Int<C>,
        ) -> Result<(), Error> {
            let r = Self::point_from_ints(in_x, in_y)? * Self::scalar_from_int(n)?;
            Self::point_to_ints(&r.into(), out_x, out_y)
        }

        fn ecdsa_sign(d: &Int<C>, m: &Int<C>, r: &mut Int<C>, s: &mut Int<C>) -> Result<(), Error> {
            let d = Self::scalar_from_int(d)?;
            let (signature, _) =
                d.try_sign_prehashed_rfc6979::<D>(m, &[]).map_err(|_| Error::world(0))?;
            r.copy_from_slice(&Self::scalar_to_int(signature.r()));
            s.copy_from_slice(&Self::scalar_to_int(signature.s()));
            Ok(())
        }

        fn ecdsa_verify(
            m: &Int<C>, x: &Int<C>, y: &Int<C>, r: &Int<C>, s: &Int<C>,
        ) -> Result<bool, Error> {
            let p = EncodedPoint::<C>::from_affine_coordinates(x, y, false);
            let p = VerifyingKey::<C>::from_encoded_point(&p).map_err(|_| Error::user(0))?;
            let signature =
                Signature::from_scalars(r.clone(), s.clone()).map_err(|_| Error::user(0))?;
            Ok(p.verify_prehash(m, &signature).is_ok())
        }
    }

    impl<C, D> Software<C, D>
    where
        C: CurveArithmetic,
        AffinePoint<C>: ToEncodedPoint<C>,
        ProjectivePoint<C>: FromEncodedPoint<C>,
        FieldBytesSize<C>: ModulusSize,
    {
        fn scalar_from_int(x: &Int<C>) -> Result<Scalar<C>, Error> {
            Ok(convert(ScalarPrimitive::from_bytes(x))?.into())
        }

        fn scalar_to_int(x: impl AsRef<Scalar<C>>) -> Int<C> {
            (*x.as_ref()).into()
        }

        fn point_from_ints(x: &Int<C>, y: &Int<C>) -> Result<ProjectivePoint<C>, Error> {
            let r = EncodedPoint::<C>::from_affine_coordinates(x, y, false);
            convert(ProjectivePoint::<C>::from_encoded_point(&r))
        }

        fn point_to_ints(p: &AffinePoint<C>, x: &mut Int<C>, y: &mut Int<C>) -> Result<(), Error> {
            let p = p.to_encoded_point(false);
            x.copy_from_slice(p.x().ok_or(Error::user(0))?);
            y.copy_from_slice(p.y().ok_or(Error::user(0))?);
            Ok(())
        }
    }

    fn convert<T>(x: CtOption<T>) -> Result<T, Error> {
        Option::<T>::from(x).ok_or(Error::user(0))
    }
}
