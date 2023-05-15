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

//! P384.

use crate::{Error, Unimplemented, Unsupported};

/// P384 interface.
pub trait Api {
    /// Returns whether P384 is supported.
    fn is_supported(&mut self) -> bool;

    /// Returns whether a scalar is valid.
    fn is_valid_scalar(&mut self, n: &[u8; 48]) -> bool;

    /// Returns whether a point is valid.
    fn is_valid_point(&mut self, x: &[u8; 48], y: &[u8; 48]) -> bool;

    /// Base point multiplication.
    fn base_point_mul(
        &mut self, n: &[u8; 48], x: &mut [u8; 48], y: &mut [u8; 48],
    ) -> Result<(), Error>;

    /// Point multiplication.
    fn point_mul(
        &mut self, n: &[u8; 48], in_x: &[u8; 48], in_y: &[u8; 48], out_x: &mut [u8; 48],
        out_y: &mut [u8; 48],
    ) -> Result<(), Error>;

    /// ECDSA signature.
    fn ecdsa_sign(
        &mut self, d: &[u8; 48], m: &[u8; 48], r: &mut [u8; 48], s: &mut [u8; 48],
    ) -> Result<(), Error>;

    /// ECDSA verification.
    fn ecdsa_verify(
        &mut self, m: &[u8; 48], x: &[u8; 48], y: &[u8; 48], r: &[u8; 48], s: &[u8; 48],
    ) -> Result<bool, Error>;
}

impl Api for Unimplemented {
    fn is_supported(&mut self) -> bool {
        unreachable!()
    }

    fn is_valid_scalar(&mut self, _: &[u8; 48]) -> bool {
        unreachable!()
    }

    fn is_valid_point(&mut self, _: &[u8; 48], _: &[u8; 48]) -> bool {
        unreachable!()
    }

    fn base_point_mul(
        &mut self, _: &[u8; 48], _: &mut [u8; 48], _: &mut [u8; 48],
    ) -> Result<(), Error> {
        unreachable!()
    }

    fn point_mul(
        &mut self, _: &[u8; 48], _: &[u8; 48], _: &[u8; 48], _: &mut [u8; 48], _: &mut [u8; 48],
    ) -> Result<(), Error> {
        unreachable!()
    }

    fn ecdsa_sign(
        &mut self, _: &[u8; 48], _: &[u8; 48], _: &mut [u8; 48], _: &mut [u8; 48],
    ) -> Result<(), Error> {
        unreachable!()
    }

    fn ecdsa_verify(
        &mut self, _: &[u8; 48], _: &[u8; 48], _: &[u8; 48], _: &[u8; 48], _: &[u8; 48],
    ) -> Result<bool, Error> {
        unreachable!()
    }
}

#[cfg(not(feature = "software-crypto-p384"))]
mod unsupported {
    use super::*;

    impl Api for Unsupported {
        fn is_supported(&mut self) -> bool {
            false
        }

        fn is_valid_scalar(&mut self, _: &[u8; 48]) -> bool {
            false
        }

        fn is_valid_point(&mut self, _: &[u8; 48], _: &[u8; 48]) -> bool {
            false
        }

        fn base_point_mul(
            &mut self, _: &[u8; 48], _: &mut [u8; 48], _: &mut [u8; 48],
        ) -> Result<(), Error> {
            Err(Error::User)
        }

        fn point_mul(
            &mut self, _: &[u8; 48], _: &[u8; 48], _: &[u8; 48], _: &mut [u8; 48], _: &mut [u8; 48],
        ) -> Result<(), Error> {
            Err(Error::User)
        }

        fn ecdsa_sign(
            &mut self, _: &[u8; 48], _: &[u8; 48], _: &mut [u8; 48], _: &mut [u8; 48],
        ) -> Result<(), Error> {
            Err(Error::User)
        }

        fn ecdsa_verify(
            &mut self, _: &[u8; 48], _: &[u8; 48], _: &[u8; 48], _: &[u8; 48], _: &[u8; 48],
        ) -> Result<bool, Error> {
            Err(Error::User)
        }
    }
}

#[cfg(feature = "software-crypto-p384")]
mod unsupported {
    use p384::ecdsa::{Signature, SigningKey, VerifyingKey};
    use p384::elliptic_curve::sec1::{FromEncodedPoint, ToEncodedPoint};
    use p384::elliptic_curve::PrimeField;
    use p384::{AffinePoint, EncodedPoint, ProjectivePoint, Scalar};
    use signature::hazmat::PrehashVerifier;

    use super::*;

    impl Api for Unsupported {
        fn is_supported(&mut self) -> bool {
            true
        }

        fn is_valid_scalar(&mut self, n: &[u8; 48]) -> bool {
            scalar_from_slice(n).is_ok()
        }

        fn is_valid_point(&mut self, x: &[u8; 48], y: &[u8; 48]) -> bool {
            point_from_slices(x, y).is_ok()
        }

        fn base_point_mul(
            &mut self, n: &[u8; 48], x: &mut [u8; 48], y: &mut [u8; 48],
        ) -> Result<(), Error> {
            let r = ProjectivePoint::GENERATOR * scalar_from_slice(n)?;
            point_to_slices(&r.to_affine(), x, y)
        }

        fn point_mul(
            &mut self, n: &[u8; 48], in_x: &[u8; 48], in_y: &[u8; 48], out_x: &mut [u8; 48],
            out_y: &mut [u8; 48],
        ) -> Result<(), Error> {
            let r = point_from_slices(in_x, in_y)? * scalar_from_slice(n)?;
            point_to_slices(&r.to_affine(), out_x, out_y)
        }

        fn ecdsa_sign(
            &mut self, d: &[u8; 48], m: &[u8; 48], r: &mut [u8; 48], s: &mut [u8; 48],
        ) -> Result<(), Error> {
            let d = SigningKey::from_bytes(d.into()).map_err(|_| Error::User)?;
            let (signature, _) = d.sign_prehash_recoverable(m).map_err(|_| Error::World)?;
            r.copy_from_slice(&signature.r().to_bytes());
            s.copy_from_slice(&signature.s().to_bytes());
            Ok(())
        }

        fn ecdsa_verify(
            &mut self, m: &[u8; 48], x: &[u8; 48], y: &[u8; 48], r: &[u8; 48], s: &[u8; 48],
        ) -> Result<bool, Error> {
            let p = EncodedPoint::from_affine_coordinates(x.into(), y.into(), false);
            let p = VerifyingKey::from_encoded_point(&p).map_err(|_| Error::User)?;
            let signature = Signature::from_scalars(*r, *s).map_err(|_| Error::User)?;
            Ok(p.verify_prehash(m, &signature).is_ok())
        }
    }

    fn scalar_from_slice(x: &[u8; 48]) -> Result<Scalar, Error> {
        Scalar::from_repr_vartime((*x).into()).ok_or(Error::User)
    }

    fn point_from_slices(x: &[u8; 48], y: &[u8; 48]) -> Result<ProjectivePoint, Error> {
        let r = EncodedPoint::from_affine_coordinates(x.into(), y.into(), false);
        let r: Option<ProjectivePoint> = ProjectivePoint::from_encoded_point(&r).into();
        r.ok_or(Error::User)
    }

    fn point_to_slices(p: &AffinePoint, x: &mut [u8; 48], y: &mut [u8; 48]) -> Result<(), Error> {
        let p = p.to_encoded_point(false);
        x.copy_from_slice(p.x().ok_or(Error::User)?);
        y.copy_from_slice(p.y().ok_or(Error::User)?);
        Ok(())
    }
}
