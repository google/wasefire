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

use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use wasefire_applet_api::crypto::ed25519 as api;
use wasefire_error::Error;
use zeroize::Zeroize;

use crate::{convert, convert_bool, convert_unit};

#[unsafe(no_mangle)]
unsafe extern "C" fn env_c2s() -> isize {
    1
}

#[unsafe(no_mangle)]
unsafe extern "C" fn env_c2l(params: api::get_layout::Params) -> isize {
    let api::get_layout::Params { kind, size, align } = params;
    let _ = api::Kind::from(kind);
    unsafe { size.write(32) };
    unsafe { align.write(1) };
    convert_unit(Ok(()))
}

#[unsafe(no_mangle)]
unsafe extern "C" fn env_c2k() -> isize {
    convert(Ok(32))
}

#[unsafe(no_mangle)]
unsafe extern "C" fn env_c2g(params: api::generate::Params) -> isize {
    let api::generate::Params { private } = params;
    let key = SigningKey::generate(&mut rand_core::OsRng).to_bytes();
    let private = unsafe { std::slice::from_raw_parts_mut(private, key.len()) };
    private.copy_from_slice(&key);
    convert_unit(Ok(()))
}

#[unsafe(no_mangle)]
unsafe extern "C" fn env_c2p(params: api::public::Params) -> isize {
    let api::public::Params { private, public } = params;
    let private = SigningKey::from_bytes(unsafe { &*private.cast() });
    let public = unsafe { std::slice::from_raw_parts_mut(public, 32) };
    public.copy_from_slice(private.verifying_key().as_bytes());
    convert_unit(Ok(()))
}

#[unsafe(no_mangle)]
unsafe extern "C" fn env_c2i(params: api::sign::Params) -> isize {
    let api::sign::Params { private, message, message_len, r, s } = params;
    let private = SigningKey::from_bytes(unsafe { &*private.cast() });
    let message = unsafe { std::slice::from_raw_parts(message, message_len) };
    let r = unsafe { std::slice::from_raw_parts_mut(r, 32) };
    let s = unsafe { std::slice::from_raw_parts_mut(s, 32) };
    let Ok(sig) = private.try_sign(message) else {
        return convert_unit(Err(Error::world(0)));
    };
    r.copy_from_slice(sig.r_bytes());
    s.copy_from_slice(sig.s_bytes());
    convert_unit(Ok(()))
}

#[unsafe(no_mangle)]
unsafe extern "C" fn env_c2v(params: api::verify::Params) -> isize {
    let api::verify::Params { public, message, message_len, r, s } = params;
    let Ok(public) = VerifyingKey::from_bytes(unsafe { &*public.cast() }) else {
        return convert_bool(Err(Error::user(0)));
    };
    let message = unsafe { std::slice::from_raw_parts(message, message_len) };
    let sig = Signature::from_components(unsafe { *r.cast() }, unsafe { *s.cast() });
    convert_bool(Ok(public.verify_strict(message, &sig).is_ok()))
}

#[unsafe(no_mangle)]
unsafe extern "C" fn env_c2d(params: api::drop::Params) -> isize {
    let api::drop::Params { private } = params;
    let private = unsafe { std::slice::from_raw_parts_mut(private, 32) };
    private.zeroize();
    convert_unit(Ok(()))
}

#[unsafe(no_mangle)]
unsafe extern "C" fn env_c2w(params: api::wrap::Params) -> isize {
    let api::wrap::Params { private, wrapped } = params;
    let private = unsafe { std::slice::from_raw_parts(private, 32) };
    let wrapped = unsafe { std::slice::from_raw_parts_mut(wrapped, 32) };
    wrapped.copy_from_slice(private);
    convert_unit(Ok(()))
}

#[unsafe(no_mangle)]
unsafe extern "C" fn env_c2u(params: api::unwrap::Params) -> isize {
    let api::unwrap::Params { wrapped, private } = params;
    let private = unsafe { std::slice::from_raw_parts_mut(private, 32) };
    let wrapped = unsafe { std::slice::from_raw_parts(wrapped, 32) };
    private.copy_from_slice(wrapped);
    convert_unit(Ok(()))
}

#[unsafe(no_mangle)]
unsafe extern "C" fn env_c2e(params: api::export::Params) -> isize {
    let api::export::Params { public, a } = params;
    let public = unsafe { std::slice::from_raw_parts(public, 32) };
    let a = unsafe { std::slice::from_raw_parts_mut(a, 32) };
    a.copy_from_slice(public);
    convert_unit(Ok(()))
}

#[unsafe(no_mangle)]
unsafe extern "C" fn env_c2m(params: api::import::Params) -> isize {
    let api::import::Params { a, public } = params;
    let public = unsafe { std::slice::from_raw_parts_mut(public, 32) };
    let a = unsafe { std::slice::from_raw_parts(a, 32) };
    public.copy_from_slice(a);
    convert_unit(Ok(()))
}
