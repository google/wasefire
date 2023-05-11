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

//! Provides elliptic curve cryptography.

use generic_array::{ArrayLength, GenericArray};
use wasefire_applet_api::crypto::ec as api;

use super::Error;

/// Computes the shared secret of an ECDH key agreement.
///
/// The curve integers are in SEC1 encoding.
pub fn ecdh<C: Curve>(
    private: &Int<C>, public_x: &Int<C>, public_y: &Int<C>, shared: &mut Int<C>,
) -> Result<(), Error> {
    let mut y = Int::<C>::default();
    C::point_mul(private, public_x, public_y, shared, &mut y)
}

/// Curve API.
///
/// This trait should only be implemented by the prelude and is thus sealed. Its purpose is to
/// provide curve-parametric algorithms.
#[sealed::sealed]
pub trait Curve {
    /// Integer type.
    type N: ArrayLength<u8>;

    /// Whether the curve is supported.
    fn is_supported() -> bool;

    // TODO: Add a is_valid_scalar.

    /// Performs base point multiplication.
    fn base_point_mul(n: &Int<Self>, x: &mut Int<Self>, y: &mut Int<Self>) -> Result<(), Error>;

    /// Performs point multiplication.
    fn point_mul(
        n: &Int<Self>, in_x: &Int<Self>, in_y: &Int<Self>, out_x: &mut Int<Self>,
        out_y: &mut Int<Self>,
    ) -> Result<(), Error>;
}

/// Array representing a curve integer in SEC1 encoding.
pub type Int<C> = GenericArray<u8, <C as Curve>::N>;

/// P-256 curve.
pub struct P256;

/// P-384 curve.
pub struct P384;

/// Internal helper trait to implement the `Curve` trait.
#[sealed::sealed]
pub trait InternalHelper {
    type N: ArrayLength<u8>;
    const C: api::Curve;
}

#[sealed::sealed]
impl InternalHelper for P256 {
    type N = typenum::consts::U32;
    const C: api::Curve = api::Curve::P256;
}

#[sealed::sealed]
impl InternalHelper for P384 {
    type N = typenum::consts::U48;
    const C: api::Curve = api::Curve::P384;
}

#[sealed::sealed]
impl<T: InternalHelper> Curve for T {
    type N = T::N;

    fn is_supported() -> bool {
        let params = api::is_supported::Params { curve: T::C as usize };
        let api::is_supported::Results { support } = unsafe { api::is_supported(params) };
        support != 0
    }

    fn base_point_mul(n: &Int<Self>, x: &mut Int<Self>, y: &mut Int<Self>) -> Result<(), Error> {
        let params = api::base_point_mul::Params {
            curve: T::C as usize,
            n: n.as_ptr(),
            x: x.as_mut_ptr(),
            y: y.as_mut_ptr(),
        };
        let api::base_point_mul::Results { res } = unsafe { api::base_point_mul(params) };
        Error::to_result(res)?;
        Ok(())
    }

    fn point_mul(
        n: &Int<Self>, in_x: &Int<Self>, in_y: &Int<Self>, out_x: &mut Int<Self>,
        out_y: &mut Int<Self>,
    ) -> Result<(), Error> {
        let params = api::point_mul::Params {
            curve: T::C as usize,
            n: n.as_ptr(),
            in_x: in_x.as_ptr(),
            in_y: in_y.as_ptr(),
            out_x: out_x.as_mut_ptr(),
            out_y: out_y.as_mut_ptr(),
        };
        let api::point_mul::Results { res } = unsafe { api::point_mul(params) };
        Error::to_result(res)?;
        Ok(())
    }
}
