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

use alloc::vec;
use alloc::vec::Vec;

use generic_array::{ArrayLength, GenericArray};
use wasefire_applet_api::crypto::ec as api;
use wasefire_applet_api::crypto::hash::Algorithm;

use super::hash::hkdf;
use super::Error;

/// ECDSA private key.
pub struct EcdsaPrivate<C: Curve>(Int<C>);

/// ECDSA public key.
pub struct EcdsaPublic<C: Curve> {
    x: Int<C>,
    y: Int<C>,
}

/// ECDSA signature.
pub struct EcdsaSignature<C: Curve> {
    r: Int<C>,
    s: Int<C>,
}

impl<C: Curve> EcdsaPrivate<C> {
    /// Returns a random (with uniform distribution) ECDSA private key.
    pub fn random() -> Result<Self, Error> {
        Ok(Self(random_private_key::<C>()?))
    }

    /// Creates a private key from its non-zero scalar SEC1 encoding.
    pub fn from_non_zero_scalar(n: Int<C>) -> Result<Self, Error> {
        ensure_non_zero_scalar::<C>(&n)?;
        Ok(Self(n))
    }

    /// Returns the public key associated to this private key.
    pub fn public_key(&self) -> EcdsaPublic<C> {
        EcdsaPublic::new(self)
    }

    /// Signs a message.
    pub fn sign(&self, message: &[u8]) -> Result<EcdsaSignature<C>, Error> {
        EcdsaSignature::new(self, message)
    }

    /// Signs a prehash.
    pub fn sign_prehash(&self, prehash: &Int<C>) -> Result<EcdsaSignature<C>, Error> {
        EcdsaSignature::new_prehash(self, prehash)
    }
}

impl<C: Curve> EcdsaPublic<C> {
    /// Returns the public key associated to a private key.
    pub fn new(private: &EcdsaPrivate<C>) -> Self {
        let mut x = Int::<C>::default();
        let mut y = Int::<C>::default();
        C::base_point_mul(&private.0, &mut x, &mut y).unwrap();
        Self { x, y }
    }

    /// Creates a public key from its coordinates in SEC1 encoding.
    pub fn from_coordinates(x: Int<C>, y: Int<C>) -> Result<Self, Error> {
        ensure_valid_point::<C>(&x, &y)?;
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

    /// Verifies the signature of a message.
    pub fn verify(&self, message: &[u8], signature: &EcdsaSignature<C>) -> Result<bool, Error> {
        signature.verify(self, message)
    }

    /// Verifies the signature of a prehash.
    pub fn verify_prehash(
        &self, prehash: &Int<C>, signature: &EcdsaSignature<C>,
    ) -> Result<bool, Error> {
        signature.verify_prehash(self, prehash)
    }
}

impl<C: Curve> EcdsaSignature<C> {
    /// Creates a signature for a message.
    pub fn new(private: &EcdsaPrivate<C>, message: &[u8]) -> Result<Self, Error> {
        Self::new_prehash(private, prehash::<C>(message)?.as_slice().into())
    }

    /// Creates a signature for a prehash.
    pub fn new_prehash(private: &EcdsaPrivate<C>, prehash: &Int<C>) -> Result<Self, Error> {
        let mut r = Int::<C>::default();
        let mut s = Int::<C>::default();
        C::ecdsa_sign(&private.0, prehash, &mut r, &mut s)?;
        Ok(Self { r, s })
    }

    /// Creates a signature from its components in SEC1 encoding.
    pub fn from_components(r: Int<C>, s: Int<C>) -> Self {
        Self { r, s }
    }

    /// Returns the r component of the signature.
    pub fn r(&self) -> &Int<C> {
        &self.r
    }

    /// Returns the s component of the signature.
    pub fn s(&self) -> &Int<C> {
        &self.s
    }

    /// Verifies a signature.
    pub fn verify(&self, public: &EcdsaPublic<C>, message: &[u8]) -> Result<bool, Error> {
        self.verify_prehash(public, prehash::<C>(message)?.as_slice().into())
    }

    /// Verifies a signature.
    pub fn verify_prehash(&self, public: &EcdsaPublic<C>, prehash: &Int<C>) -> Result<bool, Error> {
        C::ecdsa_verify(prehash, &public.x, &public.y, &self.r, &self.s)
    }
}

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
        Ok(Self(random_private_key::<C>()?))
    }

    /// Creates a private key from its non-zero scalar SEC1 encoding.
    pub fn from_non_zero_scalar(n: Int<C>) -> Result<Self, Error> {
        ensure_non_zero_scalar::<C>(&n)?;
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
        ensure_valid_point::<C>(&x, &y)?;
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

    /// Derives a key material from the shared secret using HKDF.
    pub fn derive(
        &self, algorithm: Algorithm, salt: Option<&[u8]>, info: &[u8], okm: &mut [u8],
    ) -> Result<(), Error> {
        hkdf(algorithm, salt, self.raw_bytes(), info, okm)
    }

    /// Returns the shared secret as an integer.
    ///
    /// This is not uniformly random. Prefer using [`Self::derive()`].
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

    /// Hash to use for ECDSA signatures.
    const H: Algorithm;

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

    /// Signs a message.
    fn ecdsa_sign(
        key: &Int<Self>, message: &Int<Self>, r: &mut Int<Self>, s: &mut Int<Self>,
    ) -> Result<(), Error>;

    /// Verifies a message.
    fn ecdsa_verify(
        message: &Int<Self>, x: &Int<Self>, y: &Int<Self>, r: &Int<Self>, s: &Int<Self>,
    ) -> Result<bool, Error>;
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
    const H: Algorithm;
}

#[sealed::sealed]
impl InternalHelper for P256 {
    type N = typenum::consts::U32;
    const C: api::Curve = api::Curve::P256;
    const H: Algorithm = Algorithm::Sha256;
}

#[sealed::sealed]
impl InternalHelper for P384 {
    type N = typenum::consts::U48;
    const C: api::Curve = api::Curve::P384;
    const H: Algorithm = Algorithm::Sha384;
}

#[sealed::sealed]
impl<T: InternalHelper> Curve for T {
    type N = T::N;
    const H: Algorithm = T::H;

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

    fn ecdsa_sign(
        key: &Int<Self>, message: &Int<Self>, r: &mut Int<Self>, s: &mut Int<Self>,
    ) -> Result<(), Error> {
        let params = api::ecdsa_sign::Params {
            curve: T::C as usize,
            key: key.as_ptr(),
            message: message.as_ptr(),
            r: r.as_mut_ptr(),
            s: s.as_mut_ptr(),
        };
        let api::ecdsa_sign::Results { res } = unsafe { api::ecdsa_sign(params) };
        Error::to_result(res)?;
        Ok(())
    }

    fn ecdsa_verify(
        message: &Int<Self>, x: &Int<Self>, y: &Int<Self>, r: &Int<Self>, s: &Int<Self>,
    ) -> Result<bool, Error> {
        let params = api::ecdsa_verify::Params {
            curve: T::C as usize,
            message: message.as_ptr(),
            x: x.as_ptr(),
            y: y.as_ptr(),
            r: r.as_ptr(),
            s: s.as_ptr(),
        };
        let api::ecdsa_verify::Results { res } = unsafe { api::ecdsa_verify(params) };
        Ok(Error::to_result(res)? == 1)
    }
}

fn is_zero_scalar<C: Curve>(n: &Int<C>) -> bool {
    n.iter().all(|&x| x == 0)
}

/// Returns a random (with uniform distribution) private key.
fn random_private_key<C: Curve>() -> Result<Int<C>, Error> {
    let mut scalar = Int::<C>::default();
    loop {
        // TODO(#163): Use a DRBG (possibly taking it as argument).
        crate::rng::fill_bytes(&mut scalar).map_err(|_| Error::RngFailure)?;
        if is_zero_scalar::<C>(&scalar) {
            // The probability is very low for this to happen during normal operation.
            return Err(Error::RngFailure);
        }
        if C::is_valid_scalar(&scalar) {
            return Ok(scalar);
        }
    }
}

fn ensure_non_zero_scalar<C: Curve>(n: &Int<C>) -> Result<(), Error> {
    if is_zero_scalar::<C>(n) || !C::is_valid_scalar(n) {
        return Err(Error::InvalidArgument);
    }
    Ok(())
}

fn ensure_valid_point<C: Curve>(x: &Int<C>, y: &Int<C>) -> Result<(), Error> {
    if !C::is_valid_point(x, y) {
        return Err(Error::InvalidArgument);
    }
    Ok(())
}

fn prehash<C: Curve>(message: &[u8]) -> Result<Vec<u8>, Error> {
    let mut prehash = vec![0; C::H.digest_len()];
    crate::crypto::hash::Digest::digest(C::H, message, &mut prehash)?;
    Ok(prehash)
}
