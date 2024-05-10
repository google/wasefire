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
#[cfg(any(feature = "api-crypto-hash", feature = "api-crypto-hkdf"))]
use wasefire_applet_api::crypto::hash::Algorithm;

use crate::{convert_bool, convert_unit, Error};

/// ECDSA private key.
pub struct EcdsaPrivate<C: Curve>(Private<C>);

/// ECDSA public key.
pub struct EcdsaPublic<C: Curve>(Public<C>);

/// ECDSA signature.
pub struct EcdsaSignature<C: Curve> {
    r: Int<C>,
    s: Int<C>,
}

impl<C: Curve> EcdsaPrivate<C> {
    /// Returns a random (with uniform distribution) ECDSA private key.
    #[cfg(feature = "api-rng")]
    pub fn random() -> Result<Self, Error> {
        Ok(Self(Private::random()?))
    }

    /// Creates a private key from its non-zero scalar SEC1 encoding.
    pub fn from_non_zero_scalar(n: Int<C>) -> Result<Self, Error> {
        Ok(Self(Private::from_non_zero_scalar(n)?))
    }

    /// Returns the SEC1 encoding of the private key.
    pub fn private_key(&self) -> &Int<C> {
        self.0.key()
    }

    /// Returns the public key associated to this private key.
    pub fn public_key(&self) -> EcdsaPublic<C> {
        EcdsaPublic::new(self)
    }

    /// Signs a message.
    #[cfg(feature = "api-crypto-hash")]
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
        Self(Public::new(&private.0))
    }

    /// Creates a public key from its coordinates in SEC1 encoding.
    pub fn from_coordinates(x: Int<C>, y: Int<C>) -> Result<Self, Error> {
        Ok(Self(Public::from_coordinates(x, y)?))
    }

    /// Returns the x-coordinate of the public key.
    pub fn x(&self) -> &Int<C> {
        self.0.x()
    }

    /// Returns the y-coordinate of the public key.
    pub fn y(&self) -> &Int<C> {
        self.0.y()
    }

    /// Verifies the signature of a message.
    #[cfg(feature = "api-crypto-hash")]
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
    #[cfg(feature = "api-crypto-hash")]
    pub fn new(private: &EcdsaPrivate<C>, message: &[u8]) -> Result<Self, Error> {
        Self::new_prehash(private, prehash::<C>(message)?.as_slice().into())
    }

    /// Creates a signature for a prehash.
    pub fn new_prehash(private: &EcdsaPrivate<C>, prehash: &Int<C>) -> Result<Self, Error> {
        let mut r = Int::<C>::default();
        let mut s = Int::<C>::default();
        C::ecdsa_sign(&private.0 .0, prehash, &mut r, &mut s)?;
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
    #[cfg(feature = "api-crypto-hash")]
    pub fn verify(&self, public: &EcdsaPublic<C>, message: &[u8]) -> Result<bool, Error> {
        self.verify_prehash(public, prehash::<C>(message)?.as_slice().into())
    }

    /// Verifies a signature.
    pub fn verify_prehash(&self, public: &EcdsaPublic<C>, prehash: &Int<C>) -> Result<bool, Error> {
        C::ecdsa_verify(prehash, public.x(), public.y(), &self.r, &self.s)
    }
}

/// ECDH private key.
pub struct EcdhPrivate<C: Curve>(Private<C>);

/// ECDH public key.
pub struct EcdhPublic<C: Curve>(Public<C>);

/// ECDH shared secret.
pub struct EcdhShared<C: Curve>(Int<C>);

impl<C: Curve> EcdhPrivate<C> {
    /// Returns a random (with uniform distribution) ECDH private key.
    #[cfg(feature = "api-rng")]
    pub fn random() -> Result<Self, Error> {
        Ok(Self(Private::random()?))
    }

    /// Creates a private key from its non-zero scalar SEC1 encoding.
    pub fn from_non_zero_scalar(n: Int<C>) -> Result<Self, Error> {
        Ok(Self(Private::from_non_zero_scalar(n)?))
    }

    /// Returns the SEC1 encoding of the private key.
    pub fn private_key(&self) -> &Int<C> {
        self.0.key()
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
        Self(Public::new(&private.0))
    }

    /// Creates a public key from its coordinates in SEC1 encoding.
    pub fn from_coordinates(x: Int<C>, y: Int<C>) -> Result<Self, Error> {
        Ok(Self(Public::from_coordinates(x, y)?))
    }

    /// Returns the x-coordinate of the public key.
    pub fn x(&self) -> &Int<C> {
        self.0.x()
    }

    /// Returns the y-coordinate of the public key.
    pub fn y(&self) -> &Int<C> {
        self.0.y()
    }
}

impl<C: Curve> EcdhShared<C> {
    /// Returns the shared secret associated to a private and public key.
    pub fn new(private: &EcdhPrivate<C>, public: &EcdhPublic<C>) -> Self {
        let mut x = Int::<C>::default();
        let mut y = Int::<C>::default();
        C::point_mul(&private.0 .0, public.x(), public.y(), &mut x, &mut y).unwrap();
        Self(x)
    }

    /// Derives a key material from the shared secret using HKDF.
    #[cfg(feature = "api-crypto-hkdf")]
    pub fn derive(
        &self, algorithm: Algorithm, salt: Option<&[u8]>, info: &[u8], okm: &mut [u8],
    ) -> Result<(), Error> {
        super::hash::hkdf(algorithm, salt, self.raw_bytes(), info, okm)
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
    #[cfg(feature = "api-crypto-hash")]
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
    /// Scalar size for the curve.
    type N: ArrayLength<u8>;
    /// Curve enum value.
    const C: api::Curve;
    /// Hash algorithm of the curve.
    #[cfg(feature = "api-crypto-hash")]
    const H: Algorithm;
}

#[sealed::sealed]
impl InternalHelper for P256 {
    type N = typenum::consts::U32;
    const C: api::Curve = api::Curve::P256;
    #[cfg(feature = "api-crypto-hash")]
    const H: Algorithm = Algorithm::Sha256;
}

#[sealed::sealed]
impl InternalHelper for P384 {
    type N = typenum::consts::U48;
    const C: api::Curve = api::Curve::P384;
    #[cfg(feature = "api-crypto-hash")]
    const H: Algorithm = Algorithm::Sha384;
}

#[sealed::sealed]
impl<T: InternalHelper> Curve for T {
    type N = T::N;
    #[cfg(feature = "api-crypto-hash")]
    const H: Algorithm = T::H;

    fn is_supported() -> bool {
        let params = api::is_supported::Params { curve: T::C as usize };
        convert_bool(unsafe { api::is_supported(params) }).unwrap_or(false)
    }

    fn is_valid_scalar(n: &Int<Self>) -> bool {
        let params = api::is_valid_scalar::Params { curve: T::C as usize, n: n.as_ptr() };
        convert_bool(unsafe { api::is_valid_scalar(params) }).unwrap()
    }

    fn is_valid_point(x: &Int<Self>, y: &Int<Self>) -> bool {
        let params =
            api::is_valid_point::Params { curve: T::C as usize, x: x.as_ptr(), y: y.as_ptr() };
        convert_bool(unsafe { api::is_valid_point(params) }).unwrap()
    }

    fn base_point_mul(n: &Int<Self>, x: &mut Int<Self>, y: &mut Int<Self>) -> Result<(), Error> {
        let params = api::base_point_mul::Params {
            curve: T::C as usize,
            n: n.as_ptr(),
            x: x.as_mut_ptr(),
            y: y.as_mut_ptr(),
        };
        convert_unit(unsafe { api::base_point_mul(params) })
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
        convert_unit(unsafe { api::point_mul(params) })
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
        convert_unit(unsafe { api::ecdsa_sign(params) })
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
        convert_bool(unsafe { api::ecdsa_verify(params) })
    }
}

/// Private key.
struct Private<C: Curve>(Int<C>);

/// Public key.
struct Public<C: Curve> {
    x: Int<C>,
    y: Int<C>,
}

impl<C: Curve> Private<C> {
    /// Returns a random (with uniform distribution) private key.
    #[cfg(feature = "api-rng")]
    fn random() -> Result<Self, Error> {
        let mut scalar = Int::<C>::default();
        loop {
            // TODO(#163): Use a DRBG (possibly taking it as argument).
            crate::rng::fill_bytes(&mut scalar)?;
            if is_zero_scalar::<C>(&scalar) {
                // The probability is very low for this to happen during normal operation.
                return Err(Error::world(0));
            }
            if C::is_valid_scalar(&scalar) {
                return Ok(Self(scalar));
            }
        }
    }

    /// Creates a private key from its non-zero scalar SEC1 encoding.
    fn from_non_zero_scalar(n: Int<C>) -> Result<Self, Error> {
        if is_zero_scalar::<C>(&n) || !C::is_valid_scalar(&n) {
            return Err(Error::user(0));
        }
        Ok(Self(n))
    }

    /// Returns the SEC1 encoding of the private key.
    fn key(&self) -> &Int<C> {
        &self.0
    }
}

impl<C: Curve> Public<C> {
    /// Returns the public key associated to a private key.
    fn new(private: &Private<C>) -> Self {
        let mut x = Int::<C>::default();
        let mut y = Int::<C>::default();
        C::base_point_mul(&private.0, &mut x, &mut y).unwrap();
        Self { x, y }
    }

    /// Creates a public key from its coordinates in SEC1 encoding.
    fn from_coordinates(x: Int<C>, y: Int<C>) -> Result<Self, Error> {
        if !C::is_valid_point(&x, &y) {
            return Err(Error::user(0));
        }
        Ok(Self { x, y })
    }

    /// Returns the x-coordinate of the public key.
    fn x(&self) -> &Int<C> {
        &self.x
    }

    /// Returns the y-coordinate of the public key.
    fn y(&self) -> &Int<C> {
        &self.y
    }
}

fn is_zero_scalar<C: Curve>(n: &Int<C>) -> bool {
    n.iter().all(|&x| x == 0)
}

#[cfg(feature = "api-crypto-hash")]
fn prehash<C: Curve>(message: &[u8]) -> Result<alloc::vec::Vec<u8>, Error> {
    let mut prehash = alloc::vec![0; C::H.digest_len()];
    crate::crypto::hash::Digest::digest(C::H, message, &mut prehash)?;
    Ok(prehash)
}
