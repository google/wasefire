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

//! Provides ECDH.

use alloc::alloc::{alloc, dealloc};
use core::alloc::Layout;
use core::marker::PhantomData;

use wasefire_applet_api::crypto::ecdh as api;
use wasefire_error::Code;

use crate::{Error, convert_bool, convert_unit};

/// Curve API.
#[allow(private_bounds)]
pub trait Curve: InternalCurve {
    /// Size of a field.
    const SIZE: usize;

    /// Returns whether the curve is supported.
    fn is_supported() -> bool;
}

/// P-256 curve.
pub enum P256 {}

/// P-384 curve.
pub enum P384 {}

impl<C: InternalCurve> Curve for C {
    const SIZE: usize = Self::INTERNAL_SIZE;

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

/// Shared key.
pub struct Shared<C: Curve> {
    curve: PhantomData<C>,
    object: Object,
}

impl<C: Curve> Drop for Private<C> {
    fn drop(&mut self) {
        let _ = drop_(C::CURVE, api::Kind::Private, &mut self.object);
    }
}

impl<C: Curve> Drop for Shared<C> {
    fn drop(&mut self) {
        let _ = drop_(C::CURVE, api::Kind::Shared, &mut self.object);
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

    /// Imports a private key for testing purposes.
    ///
    /// The object must be in the implementation format. It is not necessarily the scalar.
    pub fn import_testonly(object: &[u8]) -> Result<Self, Error> {
        let layout = get_layout_(C::CURVE, api::Kind::Private)?;
        Error::user(Code::InvalidLength).check(object.len() == layout.size())?;
        let mut private = Private::alloc()?;
        private.object.bytes_mut().copy_from_slice(object);
        Ok(private)
    }
}

impl<C: Curve> Public<C> {
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

impl<C: Curve> Shared<C> {
    /// Computes a shared key from a private and public key.
    pub fn new(private: &Private<C>, public: &Public<C>) -> Result<Self, Error> {
        let mut shared = Shared::alloc()?;
        shared_(C::CURVE, &private.object, &public.object, &mut shared.object)?;
        Ok(shared)
    }

    /// Exports a shared key.
    pub fn export(&self, x: &mut [u8]) -> Result<(), Error> {
        Error::user(Code::InvalidLength).check(x.len() == C::SIZE)?;
        access_(C::CURVE, &self.object, x)
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

impl<C: Curve> Shared<C> {
    fn alloc() -> Result<Self, Error> {
        let layout = get_layout_(C::CURVE, api::Kind::Shared)?;
        Ok(Shared { curve: PhantomData, object: Object::new(layout) })
    }
}

trait InternalCurve {
    const INTERNAL_SIZE: usize;
    const CURVE: api::Curve;
}

impl InternalCurve for P256 {
    const INTERNAL_SIZE: usize = 32;
    const CURVE: api::Curve = api::Curve::P256;
}

impl InternalCurve for P384 {
    const INTERNAL_SIZE: usize = 48;
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

    fn bytes_mut(&mut self) -> &mut [u8] {
        unsafe { core::slice::from_raw_parts_mut(self.data, self.layout.size()) }
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

fn generate_(curve: api::Curve, private: &mut Object) -> Result<(), Error> {
    let params = api::generate::Params { curve: curve as usize, private: private.data };
    convert_unit(unsafe { api::generate(params) })
}

fn public_(curve: api::Curve, private: &Object, public: &mut Object) -> Result<(), Error> {
    let params =
        api::public::Params { curve: curve as usize, private: private.data, public: public.data };
    convert_unit(unsafe { api::public(params) })
}

fn shared_(
    curve: api::Curve, private: &Object, public: &Object, shared: &mut Object,
) -> Result<(), Error> {
    let params = api::shared::Params {
        curve: curve as usize,
        private: private.data,
        public: public.data,
        shared: shared.data,
    };
    convert_unit(unsafe { api::shared(params) })
}

fn drop_(curve: api::Curve, kind: api::Kind, object: &mut Object) -> Result<(), Error> {
    let params =
        api::drop::Params { curve: curve as usize, kind: kind as usize, object: object.data };
    convert_unit(unsafe { api::drop(params) })
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

fn access_(curve: api::Curve, shared: &Object, x: &mut [u8]) -> Result<(), Error> {
    let params =
        api::access::Params { curve: curve as usize, shared: shared.data, x: x.as_mut_ptr() };
    convert_unit(unsafe { api::access(params) })
}
