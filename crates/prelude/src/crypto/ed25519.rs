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

//! Provides Ed25519.

use alloc::alloc::{alloc, dealloc};
use alloc::boxed::Box;
use core::alloc::Layout;

use wasefire_applet_api::crypto::ed25519 as api;
use wasefire_error::Code;

use crate::{Error, convert, convert_bool, convert_unit};

/// Returns whether Ed25519 is supported.
pub fn is_supported() -> bool {
    is_supported_()
}

/// Private key.
pub struct Private {
    object: Object,
}

/// Public key.
pub struct Public {
    object: Object,
}

impl Drop for Private {
    fn drop(&mut self) {
        let _ = drop_(&mut self.object);
    }
}

impl Private {
    /// Generates a private key.
    pub fn generate() -> Result<Self, Error> {
        let mut private = Private::alloc()?;
        generate_(&mut private.object)?;
        Ok(private)
    }

    /// Returns the public key of a private key.
    pub fn public(&self) -> Result<Public, Error> {
        let mut public = Public::alloc()?;
        public_(&self.object, &mut public.object)?;
        Ok(public)
    }

    /// Signs a message with a private key.
    pub fn sign(&self, message: &[u8], r: &mut [u8; 32], s: &mut [u8; 32]) -> Result<(), Error> {
        sign_(&self.object, message, r, s)
    }

    /// Exports a private key.
    ///
    /// This is not necessarily the scalar representing the private key. This may be a wrapped
    /// version of the private key.
    pub fn export(&self) -> Result<Box<[u8]>, Error> {
        let len = wrapped_length_()?;
        let mut wrapped = alloc::vec![0; len].into_boxed_slice();
        wrap_(&self.object, &mut wrapped)?;
        Ok(wrapped)
    }

    /// Imports a private key.
    pub fn import(wrapped: &[u8]) -> Result<Self, Error> {
        let len = wrapped_length_()?;
        Error::user(Code::InvalidLength).check(wrapped.len() == len)?;
        let mut private = Private::alloc()?;
        unwrap_(wrapped, &mut private.object)?;
        Ok(private)
    }
}

impl Public {
    /// Verifies a message with a public key.
    pub fn verify(&self, message: &[u8], r: &[u8; 32], s: &[u8; 32]) -> Result<bool, Error> {
        verify_(&self.object, message, r, s)
    }

    /// Exports a public key.
    pub fn export(&self, a: &mut [u8; 32]) -> Result<(), Error> {
        export_(&self.object, a)
    }

    /// Imports a public key.
    pub fn import(a: &[u8; 32]) -> Result<Self, Error> {
        let mut public = Public::alloc()?;
        import_(a, &mut public.object)?;
        Ok(public)
    }
}

impl Private {
    fn alloc() -> Result<Self, Error> {
        let layout = get_layout_(api::Kind::Private)?;
        Ok(Private { object: Object::new(layout)? })
    }
}

impl Public {
    fn alloc() -> Result<Self, Error> {
        let layout = get_layout_(api::Kind::Public)?;
        Ok(Public { object: Object::new(layout)? })
    }
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
    fn new(layout: Layout) -> Result<Self, Error> {
        let data = unsafe { alloc(layout) };
        Error::world(Code::NotEnough).check(!data.is_null())?;
        Ok(Object { layout, data })
    }
}

fn is_supported_() -> bool {
    convert_bool(unsafe { api::is_supported() }).unwrap_or(false)
}

fn get_layout_(kind: api::Kind) -> Result<Layout, Error> {
    let mut size = 0u32;
    let mut align = 0u32;
    let params =
        api::get_layout::Params { kind: kind as usize, size: &mut size, align: &mut align };
    convert_unit(unsafe { api::get_layout(params) })?;
    Layout::from_size_align(size as usize, align as usize).map_err(|_| Error::world(0))
}

fn wrapped_length_() -> Result<usize, Error> {
    convert(unsafe { api::wrapped_length() })
}

fn generate_(private: &mut Object) -> Result<(), Error> {
    let params = api::generate::Params { private: private.data };
    convert_unit(unsafe { api::generate(params) })
}

fn public_(private: &Object, public: &mut Object) -> Result<(), Error> {
    let params = api::public::Params { private: private.data, public: public.data };
    convert_unit(unsafe { api::public(params) })
}

fn sign_(
    private: &Object, message: &[u8], r: &mut [u8; 32], s: &mut [u8; 32],
) -> Result<(), Error> {
    let params = api::sign::Params {
        private: private.data,
        message: message.as_ptr(),
        message_len: message.len(),
        r: r.as_mut_ptr(),
        s: s.as_mut_ptr(),
    };
    convert_unit(unsafe { api::sign(params) })
}

fn verify_(public: &Object, message: &[u8], r: &[u8; 32], s: &[u8; 32]) -> Result<bool, Error> {
    let params = api::verify::Params {
        public: public.data,
        message: message.as_ptr(),
        message_len: message.len(),
        r: r.as_ptr(),
        s: s.as_ptr(),
    };
    convert_bool(unsafe { api::verify(params) })
}

fn drop_(private: &mut Object) -> Result<(), Error> {
    let params = api::drop::Params { private: private.data };
    convert_unit(unsafe { api::drop(params) })
}

fn wrap_(private: &Object, wrapped: &mut [u8]) -> Result<(), Error> {
    let params = api::wrap::Params { private: private.data, wrapped: wrapped.as_mut_ptr() };
    convert_unit(unsafe { api::wrap(params) })
}

fn unwrap_(wrapped: &[u8], private: &mut Object) -> Result<(), Error> {
    let params = api::unwrap::Params { wrapped: wrapped.as_ptr(), private: private.data };
    convert_unit(unsafe { api::unwrap(params) })
}

fn export_(public: &Object, a: &mut [u8; 32]) -> Result<(), Error> {
    let params = api::export::Params { public: public.data, a: a.as_mut_ptr() };
    convert_unit(unsafe { api::export(params) })
}

fn import_(a: &[u8; 32], public: &mut Object) -> Result<(), Error> {
    let params = api::import::Params { a: a.as_ptr(), public: public.data };
    convert_unit(unsafe { api::import(params) })
}
