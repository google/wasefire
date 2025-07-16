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

use digest::typenum::Unsigned;
use elliptic_curve::sec1::{EncodedPoint, FromEncodedPoint, ModulusSize, ToEncodedPoint};
use elliptic_curve::subtle::CtOption;
use elliptic_curve::zeroize::Zeroize;
use elliptic_curve::{
    AffinePoint, CurveArithmetic, FieldBytes, FieldBytesSize, ProjectivePoint, Scalar,
    ScalarPrimitive, SecretKey,
};
use wasefire_applet_api::crypto::ecdh as api;
use wasefire_error::{Code, Error};

use crate::convert_unit;

#[unsafe(no_mangle)]
unsafe extern "C" fn env_cis(_: api::is_supported::Params) -> isize {
    1
}

#[unsafe(no_mangle)]
unsafe extern "C" fn env_cil(params: api::get_layout::Params) -> isize {
    let api::get_layout::Params { curve, kind, size, align } = params;
    let base = match api::Curve::from(curve) {
        api::Curve::P256 => 32,
        api::Curve::P384 => 48,
    };
    let factor = match api::Kind::from(kind) {
        api::Kind::Private => 1,
        api::Kind::Public => 2,
        api::Kind::Shared => 1,
    };
    unsafe { size.write(base * factor) };
    unsafe { align.write(1) };
    convert_unit(Ok(()))
}

#[unsafe(no_mangle)]
unsafe extern "C" fn env_cig(params: api::generate::Params) -> isize {
    let api::generate::Params { curve, private } = params;
    match api::Curve::from(curve) {
        api::Curve::P256 => generate::<p256::NistP256>(private),
        api::Curve::P384 => generate::<p384::NistP384>(private),
    };
    convert_unit(Ok(()))
}

fn generate<C: CurveArithmetic>(private: *mut u8) {
    let key = SecretKey::<C>::random(&mut rand_core::OsRng).to_bytes();
    let private = unsafe { std::slice::from_raw_parts_mut(private, key.len()) };
    private.copy_from_slice(&key);
}

#[unsafe(no_mangle)]
unsafe extern "C" fn env_cip(params: api::public::Params) -> isize {
    let api::public::Params { curve, private, public } = params;
    let res = match api::Curve::from(curve) {
        api::Curve::P256 => public_key::<p256::NistP256>(private, public),
        api::Curve::P384 => public_key::<p384::NistP384>(private, public),
    };
    convert_unit(res)
}

fn public_key<C>(private: *const u8, public: *mut u8) -> Result<(), Error>
where
    C: CurveArithmetic,
    AffinePoint<C>: ToEncodedPoint<C>,
    FieldBytesSize<C>: ModulusSize,
{
    let n = FieldBytesSize::<C>::USIZE;
    let private = unsafe { std::slice::from_raw_parts(private, n) };
    let key = SecretKey::<C>::from_bytes(FieldBytes::<C>::from_slice(private))
        .map_err(|_| Error::user(Code::InvalidArgument))?;
    let key = key.public_key().as_affine().to_encoded_point(false);
    let public = unsafe { std::slice::from_raw_parts_mut(public, 2 * n) };
    let (x, y) = public.split_at_mut(n);
    x.copy_from_slice(key.x().ok_or(Error::user(0))?);
    y.copy_from_slice(key.y().ok_or(Error::user(0))?);
    Ok(())
}

#[unsafe(no_mangle)]
unsafe extern "C" fn env_cia(params: api::shared::Params) -> isize {
    let api::shared::Params { curve, private, public, shared } = params;
    let res = match api::Curve::from(curve) {
        api::Curve::P256 => shared_::<p256::NistP256>(private, public, shared),
        api::Curve::P384 => shared_::<p384::NistP384>(private, public, shared),
    };
    convert_unit(res)
}

fn shared_<C>(private: *const u8, public: *const u8, shared: *mut u8) -> Result<(), Error>
where
    C: CurveArithmetic,
    AffinePoint<C>: ToEncodedPoint<C>,
    ProjectivePoint<C>: FromEncodedPoint<C>,
    FieldBytesSize<C>: ModulusSize,
{
    let n = FieldBytesSize::<C>::USIZE;
    let private = unsafe { std::slice::from_raw_parts(private, n) };
    let private = unwrap_ct(ScalarPrimitive::<C>::from_bytes(private.into()))?;
    let private = Scalar::<C>::from(private);
    let public = unsafe { std::slice::from_raw_parts(public, 2 * n) };
    let (x, y) = public.split_at(n);
    let public = EncodedPoint::<C>::from_affine_coordinates(x.into(), y.into(), false);
    let public = unwrap_ct(ProjectivePoint::<C>::from_encoded_point(&public))?;
    let secret: AffinePoint<C> = (public * private).into();
    let secret = secret.to_encoded_point(false);
    let shared = unsafe { std::slice::from_raw_parts_mut(shared, n) };
    shared.copy_from_slice(secret.x().ok_or(Error::user(0))?);
    Ok(())
}

#[unsafe(no_mangle)]
unsafe extern "C" fn env_cid(params: api::drop::Params) -> isize {
    let api::drop::Params { curve, kind, object } = params;
    let n = match api::Curve::from(curve) {
        api::Curve::P256 => 32,
        api::Curve::P384 => 48,
    };
    match api::Kind::from(kind) {
        api::Kind::Private => (),
        api::Kind::Public => return convert_unit(Err(Error::user(Code::InvalidArgument))),
        api::Kind::Shared => (),
    }
    let object = unsafe { std::slice::from_raw_parts_mut(object, n) };
    object.zeroize();
    convert_unit(Ok(()))
}

#[unsafe(no_mangle)]
unsafe extern "C" fn env_cie(params: api::export::Params) -> isize {
    let api::export::Params { curve, public, x, y } = params;
    let n = match api::Curve::from(curve) {
        api::Curve::P256 => 32,
        api::Curve::P384 => 48,
    };
    let public = unsafe { std::slice::from_raw_parts(public, 2 * n) };
    let x = unsafe { std::slice::from_raw_parts_mut(x, n) };
    let y = unsafe { std::slice::from_raw_parts_mut(y, n) };
    x.copy_from_slice(&public[.. n]);
    y.copy_from_slice(&public[n ..]);
    convert_unit(Ok(()))
}

#[unsafe(no_mangle)]
unsafe extern "C" fn env_cii(params: api::import::Params) -> isize {
    let api::import::Params { curve, x, y, public } = params;
    let n = match api::Curve::from(curve) {
        api::Curve::P256 => 32,
        api::Curve::P384 => 48,
    };
    let public = unsafe { std::slice::from_raw_parts_mut(public, 2 * n) };
    let x = unsafe { std::slice::from_raw_parts(x, n) };
    let y = unsafe { std::slice::from_raw_parts(y, n) };
    public[.. n].copy_from_slice(x);
    public[n ..].copy_from_slice(y);
    convert_unit(Ok(()))
}

#[unsafe(no_mangle)]
unsafe extern "C" fn env_cix(params: api::access::Params) -> isize {
    let api::access::Params { curve, shared, x } = params;
    let n = match api::Curve::from(curve) {
        api::Curve::P256 => 32,
        api::Curve::P384 => 48,
    };
    let shared = unsafe { std::slice::from_raw_parts(shared, n) };
    let x = unsafe { std::slice::from_raw_parts_mut(x, n) };
    x.copy_from_slice(shared);
    convert_unit(Ok(()))
}

fn unwrap_ct<T>(x: CtOption<T>) -> Result<T, Error> {
    Option::<T>::from(x).ok_or(Error::user(0))
}
