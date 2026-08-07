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

use crypto_common::array::ArraySize;
#[cfg(feature = "internal-software-crypto-ecc")]
pub use software::*;

use crate::{Error, Support};

/// Elliptic-curve cryptography interface.
pub trait Api<N: ArraySize>: Support<bool> + Send {
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
pub type Int<N> = crypto_common::array::Array<u8, N>;

#[cfg(feature = "internal-software-crypto-ecc")]
mod software {
    use core::marker::PhantomData;

    use crypto_common::BlockSizeUser;
    use ecdsa::{EcdsaCurve, PrimeCurve, Signature, SignatureSize, SigningKey, VerifyingKey};
    use elliptic_curve::ff::PrimeField;
    use elliptic_curve::sec1::{FromSec1Point, ModulusSize, Sec1Point, ToSec1Point};
    use elliptic_curve::subtle::CtOption;
    use elliptic_curve::{
        AffinePoint, CurveArithmetic, FieldBytesSize, Group, NonZeroScalar, ProjectivePoint, Scalar,
    };
    use signature::digest::{Digest, FixedOutput, FixedOutputReset};
    use signature::hazmat::{PrehashSigner, PrehashVerifier};

    use super::*;
    use crate::Support;

    /// Generic elliptic-curve software implementation.
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
        C: Send + PrimeCurve + CurveArithmetic + EcdsaCurve,
        D: Support<bool> + Send,
        D: Digest + BlockSizeUser + FixedOutput<OutputSize = FieldBytesSize<C>> + FixedOutputReset,
        AffinePoint<C>: FromSec1Point<C> + ToSec1Point<C>,
        ProjectivePoint<C>: FromSec1Point<C>,
        SignatureSize<C>: ArraySize,
        FieldBytesSize<C>: ModulusSize,
        SigningKey<C>: PrehashSigner<Signature<C>>,
    {
        fn is_valid_scalar(n: &Int<C>) -> bool {
            Self::scalar_from_int(n).is_ok()
        }

        fn is_valid_point(x: &Int<C>, y: &Int<C>) -> bool {
            Self::point_from_ints(x, y).is_ok()
        }

        fn base_point_mul(n: &Int<C>, x: &mut Int<C>, y: &mut Int<C>) -> Result<(), Error> {
            let r = ProjectivePoint::<C>::generator() * Self::scalar_from_int(n)?;
            Self::point_to_ints(&r.into(), x, y)
        }

        fn point_mul(
            n: &Int<C>, in_x: &Int<C>, in_y: &Int<C>, out_x: &mut Int<C>, out_y: &mut Int<C>,
        ) -> Result<(), Error> {
            let r = Self::point_from_ints(in_x, in_y)? * Self::scalar_from_int(n)?;
            Self::point_to_ints(&r.into(), out_x, out_y)
        }

        fn ecdsa_sign(d: &Int<C>, m: &Int<C>, r: &mut Int<C>, s: &mut Int<C>) -> Result<(), Error> {
            let d = convert(NonZeroScalar::<C>::new(Self::scalar_from_int(d)?))?;
            let signature =
                SigningKey::<C>::from(d).sign_prehash(m).map_err(|_| Error::world(0))?;
            r.copy_from_slice(&Self::scalar_to_int(signature.r()));
            s.copy_from_slice(&Self::scalar_to_int(signature.s()));
            Ok(())
        }

        fn ecdsa_verify(
            m: &Int<C>, x: &Int<C>, y: &Int<C>, r: &Int<C>, s: &Int<C>,
        ) -> Result<bool, Error> {
            let p = Sec1Point::<C>::from_affine_coordinates(x, y, false);
            let p = VerifyingKey::<C>::from_sec1_point(&p).map_err(|_| Error::user(0))?;
            let signature = Signature::from_scalars(*r, *s).map_err(|_| Error::user(0))?;
            Ok(p.verify_prehash(m, &signature).is_ok())
        }
    }

    impl<C, D> Software<C, D>
    where
        C: CurveArithmetic,
        AffinePoint<C>: ToSec1Point<C>,
        ProjectivePoint<C>: FromSec1Point<C>,
        FieldBytesSize<C>: ModulusSize,
    {
        fn scalar_from_int(x: &Int<C>) -> Result<Scalar<C>, Error> {
            convert(Scalar::<C>::from_repr(*x))
        }

        fn scalar_to_int(x: impl AsRef<Scalar<C>>) -> Int<C> {
            (*x.as_ref()).into()
        }

        fn point_from_ints(x: &Int<C>, y: &Int<C>) -> Result<ProjectivePoint<C>, Error> {
            let r = Sec1Point::<C>::from_affine_coordinates(x, y, false);
            convert(ProjectivePoint::<C>::from_sec1_point(&r).into())
        }

        fn point_to_ints(p: &AffinePoint<C>, x: &mut Int<C>, y: &mut Int<C>) -> Result<(), Error> {
            let p = p.to_sec1_point(false);
            x.copy_from_slice(p.x().ok_or(Error::user(0))?);
            y.copy_from_slice(p.y().ok_or(Error::user(0))?);
            Ok(())
        }
    }

    fn convert<T>(x: CtOption<T>) -> Result<T, Error> {
        Option::<T>::from(x).ok_or(Error::user(0))
    }
}
