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

/// ECDH private key.
pub struct EcdhPrivate<C: Curve>(Int<C>);

/// ECDH public key.
pub struct EcdhPublic<C: Curve> {
    x: Int<C>,
    y: Int<C>,
}

/// ECDH shared secret.
pub struct EcdhShared<C: Curve>(Int<C>);

impl<C: Curve> EcdhPrivate<C> {
    /// Returns a random (with uniform distribution) ECDH private key.
    pub fn random() -> Result<Self, Error> {
        let mut scalar = Int::<C>::default();
        loop {
            crate::rng::fill_bytes(&mut scalar).map_err(|_| Error::RngFailure)?;
            if is_zero_scalar::<C>(&scalar) {
                // The probability is very low for this to happen during normal operation.
                return Err(Error::RngFailure);
            }
            if C::is_valid_scalar(&scalar) {
                return Ok(Self(scalar));
            }
        }
    }

    /// Creates a private key from its non-zero scalar SEC1 encoding.
    pub fn from_non_zero_scalar(n: Int<C>) -> Result<Self, Error> {
        if is_zero_scalar::<C>(&n) || !C::is_valid_scalar(&n) {
            return Err(Error::InvalidArgument);
        }
        Ok(Self(n))
    }

    /// Returns the public key associated to this private key.
    pub fn public_key(&self) -> EcdhPublic<C> {
        EcdhPublic::new(self)
    }

    /// Returns the shared secret associated to this private key and a public key.
    pub fn diffie_hellman(&self, public: &EcdhPublic<C>) -> EcdhShared<C> {
        EcdhShared::new(self, public)
    }
}

impl<C: Curve> EcdhPublic<C> {
    /// Returns the public key associated to a private key.
    pub fn new(private: &EcdhPrivate<C>) -> Self {
        let mut x = Int::<C>::default();
        let mut y = Int::<C>::default();
        C::base_point_mul(&private.0, &mut x, &mut y).unwrap();
        Self { x, y }
    }

    /// Creates a public key from its coordinates in SEC1 encoding.
    pub fn from_coordinates(x: Int<C>, y: Int<C>) -> Result<Self, Error> {
        if !C::is_valid_point(&x, &y) {
            return Err(Error::InvalidArgument);
        }
        Ok(Self { x, y })
    }

    /// Returns the x-coordinate of the public key.
    pub fn x(&self) -> &Int<C> {
        &self.x
    }

    /// Returns the y-coordinate of the public key.
    pub fn y(&self) -> &Int<C> {
        &self.y
    }
}

impl<C: Curve> EcdhShared<C> {
    /// Returns the shared secret associated to a private and public key.
    pub fn new(private: &EcdhPrivate<C>, public: &EcdhPublic<C>) -> Self {
        let mut x = Int::<C>::default();
        let mut y = Int::<C>::default();
        C::point_mul(&private.0, &public.x, &public.y, &mut x, &mut y).unwrap();
        Self(x)
    }

    /// Returns the shared secret as an integer.
    ///
    /// This is not uniformly random. An HKDF should be used to extract entropy from this shared
    /// secret.
    pub fn raw_bytes(&self) -> &Int<C> {
        &self.0
    }
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

    /// Whether a scalar is valid, i.e. smaller than the field's modulus.
    fn is_valid_scalar(n: &Int<Self>) -> bool;

    /// Whether a point given by coordinates is valid.
    fn is_valid_point(x: &Int<Self>, y: &Int<Self>) -> bool;

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

    fn is_valid_scalar(n: &Int<Self>) -> bool {
        let params = api::is_valid_scalar::Params { curve: T::C as usize, n: n.as_ptr() };
        let api::is_valid_scalar::Results { valid } = unsafe { api::is_valid_scalar(params) };
        valid != 0
    }

    fn is_valid_point(x: &Int<Self>, y: &Int<Self>) -> bool {
        let params =
            api::is_valid_point::Params { curve: T::C as usize, x: x.as_ptr(), y: y.as_ptr() };
        let api::is_valid_point::Results { valid } = unsafe { api::is_valid_point(params) };
        valid != 0
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

fn is_zero_scalar<C: Curve>(n: &Int<C>) -> bool {
    n.iter().all(|&x| x == 0)
}
