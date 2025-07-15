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

//! Provides ECDSA.

use alloc::alloc::{alloc, dealloc};
use alloc::boxed::Box;
use core::alloc::Layout;
use core::marker::PhantomData;

use wasefire_applet_api::crypto::ecdsa as api;
#[cfg(feature = "api-crypto-hash")]
use wasefire_applet_api::crypto::hash::Algorithm;
use wasefire_error::Code;

use crate::{Error, convert, convert_bool, convert_unit};

/// Curve API.
#[allow(private_bounds)]
pub trait Curve: InternalCurve {
    /// Size of a field.
    const SIZE: usize;

    /// How to hash messages.
    #[cfg(feature = "api-crypto-hash")]
    const HASH: Algorithm;

    /// Returns whether the curve is supported.
    fn is_supported() -> bool;
}

/// P-256 curve.
pub enum P256 {}

/// P-384 curve.
pub enum P384 {}

impl<C: InternalCurve> Curve for C {
    const SIZE: usize = Self::INTERNAL_SIZE;

    #[cfg(feature = "api-crypto-hash")]
    const HASH: Algorithm = Self::INTERNAL_HASH;

    fn is_supported() -> bool {
        is_supported_(Self::CURVE)
    }
}

/// Private key.
pub struct Private<C: Curve> {
    curve: PhantomData<C>,
    object: Object,
}

/// Public key.
pub struct Public<C: Curve> {
    curve: PhantomData<C>,
    object: Object,
}

impl<C: Curve> Drop for Private<C> {
    fn drop(&mut self) {
        let _ = drop_(C::CURVE, &mut self.object);
    }
}

impl<C: Curve> Private<C> {
    /// Generates a private key.
    pub fn generate() -> Result<Self, Error> {
        let mut private = Private::alloc()?;
        generate_(C::CURVE, &mut private.object)?;
        Ok(private)
    }

    /// Returns the public key of a private key.
    pub fn public(&self) -> Result<Public<C>, Error> {
        let mut public = Public::alloc()?;
        public_(C::CURVE, &self.object, &mut public.object)?;
        Ok(public)
    }

    /// Signs a message with a private key.
    #[cfg(feature = "api-crypto-hash")]
    pub fn sign(&self, message: &[u8], r: &mut [u8], s: &mut [u8]) -> Result<(), Error> {
        let mut digest = alloc::vec![0; C::SIZE];
        crate::crypto::hash::Digest::digest(C::HASH, message, &mut digest)?;
        self.sign_prehash(&digest, r, s)
    }

    /// Signs a pre-hashed message with a private key.
    pub fn sign_prehash(&self, digest: &[u8], r: &mut [u8], s: &mut [u8]) -> Result<(), Error> {
        Error::user(Code::InvalidLength).check(digest.len() == C::SIZE)?;
        Error::user(Code::InvalidLength).check(r.len() == C::SIZE)?;
        Error::user(Code::InvalidLength).check(s.len() == C::SIZE)?;
        sign_(C::CURVE, &self.object, digest, r, s)
    }

    /// Exports a private key.
    ///
    /// This is not necessarily the scalar representing the private key. This may be a wrapped
    /// version of the private key.
    pub fn export(&self) -> Result<Box<[u8]>, Error> {
        let len = wrapped_length_(C::CURVE)?;
        let mut wrapped = alloc::vec![0; len].into_boxed_slice();
        wrap_(C::CURVE, &self.object, &mut wrapped)?;
        Ok(wrapped)
    }

    /// Imports a private key.
    pub fn import(wrapped: &[u8]) -> Result<Self, Error> {
        let len = wrapped_length_(C::CURVE)?;
        Error::user(Code::InvalidLength).check(wrapped.len() == len)?;
        let mut private = Private::alloc()?;
        unwrap_(C::CURVE, wrapped, &mut private.object)?;
        Ok(private)
    }
}

impl<C: Curve> Public<C> {
    /// Verifies a message with a public key.
    #[cfg(feature = "api-crypto-hash")]
    pub fn verify(&self, message: &[u8], r: &[u8], s: &[u8]) -> Result<bool, Error> {
        let mut digest = alloc::vec![0; C::SIZE];
        crate::crypto::hash::Digest::digest(C::HASH, message, &mut digest)?;
        self.verify_prehash(&digest, r, s)
    }

    /// Verifies a pre-hashed message with a public key.
    pub fn verify_prehash(&self, digest: &[u8], r: &[u8], s: &[u8]) -> Result<bool, Error> {
        Error::user(Code::InvalidLength).check(digest.len() == C::SIZE)?;
        Error::user(Code::InvalidLength).check(r.len() == C::SIZE)?;
        Error::user(Code::InvalidLength).check(s.len() == C::SIZE)?;
        verify_(C::CURVE, &self.object, digest, r, s)
    }

    /// Exports a public key.
    pub fn export(&self, x: &mut [u8], y: &mut [u8]) -> Result<(), Error> {
        Error::user(Code::InvalidLength).check(x.len() == C::SIZE)?;
        Error::user(Code::InvalidLength).check(y.len() == C::SIZE)?;
        export_(C::CURVE, &self.object, x, y)
    }

    /// Imports a public key.
    pub fn import(x: &[u8], y: &[u8]) -> Result<Self, Error> {
        Error::user(Code::InvalidLength).check(x.len() == C::SIZE)?;
        Error::user(Code::InvalidLength).check(y.len() == C::SIZE)?;
        let mut public = Public::alloc()?;
        import_(C::CURVE, x, y, &mut public.object)?;
        Ok(public)
    }
}

impl<C: Curve> Private<C> {
    fn alloc() -> Result<Self, Error> {
        let layout = get_layout_(C::CURVE, api::Kind::Private)?;
        Ok(Private { curve: PhantomData, object: Object::new(layout) })
    }
}

impl<C: Curve> Public<C> {
    fn alloc() -> Result<Self, Error> {
        let layout = get_layout_(C::CURVE, api::Kind::Public)?;
        Ok(Public { curve: PhantomData, object: Object::new(layout) })
    }
}

trait InternalCurve {
    const INTERNAL_SIZE: usize;
    #[cfg(feature = "api-crypto-hash")]
    const INTERNAL_HASH: Algorithm;
    const CURVE: api::Curve;
}

impl InternalCurve for P256 {
    const INTERNAL_SIZE: usize = 32;
    #[cfg(feature = "api-crypto-hash")]
    const INTERNAL_HASH: Algorithm = Algorithm::Sha256;
    const CURVE: api::Curve = api::Curve::P256;
}

impl InternalCurve for P384 {
    const INTERNAL_SIZE: usize = 48;
    #[cfg(feature = "api-crypto-hash")]
    const INTERNAL_HASH: Algorithm = Algorithm::Sha384;
    const CURVE: api::Curve = api::Curve::P384;
}

struct Object {
    layout: Layout,
    data: *mut u8,
}

impl Drop for Object {
    fn drop(&mut self) {
        unsafe { dealloc(self.data, self.layout) };
    }
}

impl Object {
    fn new(layout: Layout) -> Self {
        Object { layout, data: unsafe { alloc(layout) } }
    }
}

fn is_supported_(curve: api::Curve) -> bool {
    let params = api::is_supported::Params { curve: curve as usize };
    convert_bool(unsafe { api::is_supported(params) }).unwrap_or(false)
}

fn get_layout_(curve: api::Curve, kind: api::Kind) -> Result<Layout, Error> {
    let mut size = 0u32;
    let mut align = 0u32;
    let params = api::get_layout::Params {
        curve: curve as usize,
        kind: kind as usize,
        size: &mut size,
        align: &mut align,
    };
    convert_unit(unsafe { api::get_layout(params) })?;
    Layout::from_size_align(size as usize, align as usize).map_err(|_| Error::world(0))
}

fn wrapped_length_(curve: api::Curve) -> Result<usize, Error> {
    let params = api::wrapped_length::Params { curve: curve as usize };
    convert(unsafe { api::wrapped_length(params) })
}

fn generate_(curve: api::Curve, private: &mut Object) -> Result<(), Error> {
    let params = api::generate::Params { curve: curve as usize, private: private.data };
    convert_unit(unsafe { api::generate(params) })
}

fn public_(curve: api::Curve, private: &Object, public: &mut Object) -> Result<(), Error> {
    let params =
        api::public::Params { curve: curve as usize, private: private.data, public: public.data };
    convert_unit(unsafe { api::public(params) })
}

fn sign_(
    curve: api::Curve, private: &Object, digest: &[u8], r: &mut [u8], s: &mut [u8],
) -> Result<(), Error> {
    let params = api::sign::Params {
        curve: curve as usize,
        private: private.data,
        digest: digest.as_ptr(),
        r: r.as_mut_ptr(),
        s: s.as_mut_ptr(),
    };
    convert_unit(unsafe { api::sign(params) })
}

fn verify_(
    curve: api::Curve, public: &Object, digest: &[u8], r: &[u8], s: &[u8],
) -> Result<bool, Error> {
    let params = api::verify::Params {
        curve: curve as usize,
        public: public.data,
        digest: digest.as_ptr(),
        r: r.as_ptr(),
        s: s.as_ptr(),
    };
    convert_bool(unsafe { api::verify(params) })
}

fn drop_(curve: api::Curve, private: &mut Object) -> Result<(), Error> {
    let params = api::drop::Params { curve: curve as usize, private: private.data };
    convert_unit(unsafe { api::drop(params) })
}

fn wrap_(curve: api::Curve, private: &Object, wrapped: &mut [u8]) -> Result<(), Error> {
    let params = api::wrap::Params {
        curve: curve as usize,
        private: private.data,
        wrapped: wrapped.as_mut_ptr(),
    };
    convert_unit(unsafe { api::wrap(params) })
}

fn unwrap_(curve: api::Curve, wrapped: &[u8], private: &mut Object) -> Result<(), Error> {
    let params = api::unwrap::Params {
        curve: curve as usize,
        wrapped: wrapped.as_ptr(),
        private: private.data,
    };
    convert_unit(unsafe { api::unwrap(params) })
}

fn export_(curve: api::Curve, public: &Object, x: &mut [u8], y: &mut [u8]) -> Result<(), Error> {
    let params = api::export::Params {
        curve: curve as usize,
        public: public.data,
        x: x.as_mut_ptr(),
        y: y.as_mut_ptr(),
    };
    convert_unit(unsafe { api::export(params) })
}

fn import_(curve: api::Curve, x: &[u8], y: &[u8], public: &mut Object) -> Result<(), Error> {
    let params = api::import::Params {
        curve: curve as usize,
        x: x.as_ptr(),
        y: y.as_ptr(),
        public: public.data,
    };
    convert_unit(unsafe { api::import(params) })
}
