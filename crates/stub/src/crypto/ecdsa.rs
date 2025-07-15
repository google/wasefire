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

use crypto_common::BlockSizeUser;
use crypto_common::generic_array::ArrayLength;
use digest::typenum::Unsigned;
use digest::{Digest, FixedOutput, FixedOutputReset};
use ecdsa::hazmat::{SignPrimitive, VerifyPrimitive};
use ecdsa::{Signature, SignatureSize, SigningKey, VerifyingKey};
use elliptic_curve::sec1::{EncodedPoint, FromEncodedPoint, ModulusSize, ToEncodedPoint};
use elliptic_curve::zeroize::Zeroize;
use elliptic_curve::{
    AffinePoint, CurveArithmetic, FieldBytes, FieldBytesSize, PrimeCurve, Scalar,
};
use signature::hazmat::PrehashVerifier;
use wasefire_applet_api::crypto::ecdsa as api;
use wasefire_error::{Code, Error};

use crate::{convert, convert_bool, convert_unit};

#[unsafe(no_mangle)]
unsafe extern "C" fn env_cds(_: api::is_supported::Params) -> isize {
    1
}

#[unsafe(no_mangle)]
unsafe extern "C" fn env_cdl(params: api::get_layout::Params) -> isize {
    let api::get_layout::Params { curve, kind, size, align } = params;
    let base = match api::Curve::from(curve) {
        api::Curve::P256 => 32,
        api::Curve::P384 => 48,
    };
    let factor = match api::Kind::from(kind) {
        api::Kind::Private => 1,
        api::Kind::Public => 2,
    };
    unsafe { size.write(base * factor) };
    unsafe { align.write(1) };
    convert_unit(Ok(()))
}

#[unsafe(no_mangle)]
unsafe extern "C" fn env_cdk(params: api::wrapped_length::Params) -> isize {
    let api::wrapped_length::Params { curve } = params;
    let size = match api::Curve::from(curve) {
        api::Curve::P256 => 32,
        api::Curve::P384 => 48,
    };
    convert(Ok(size))
}

#[unsafe(no_mangle)]
unsafe extern "C" fn env_cdg(params: api::generate::Params) -> isize {
    let api::generate::Params { curve, private } = params;
    match api::Curve::from(curve) {
        api::Curve::P256 => generate::<p256::NistP256>(private),
        api::Curve::P384 => generate::<p384::NistP384>(private),
    };
    convert_unit(Ok(()))
}

fn generate<C>(private: *mut u8)
where
    C: PrimeCurve + CurveArithmetic,
    Scalar<C>: SignPrimitive<C>,
    SignatureSize<C>: ArrayLength<u8>,
{
    let key = SigningKey::<C>::random(&mut rand_core::OsRng).to_bytes();
    let private = unsafe { std::slice::from_raw_parts_mut(private, key.len()) };
    private.copy_from_slice(&key);
}

#[unsafe(no_mangle)]
unsafe extern "C" fn env_cdp(params: api::public::Params) -> isize {
    let api::public::Params { curve, private, public } = params;
    let res = match api::Curve::from(curve) {
        api::Curve::P256 => public_key::<p256::NistP256>(private, public),
        api::Curve::P384 => public_key::<p384::NistP384>(private, public),
    };
    convert_unit(res)
}

fn public_key<C>(private: *const u8, public: *mut u8) -> Result<(), Error>
where
    C: PrimeCurve + CurveArithmetic,
    Scalar<C>: SignPrimitive<C>,
    SignatureSize<C>: ArrayLength<u8>,
    AffinePoint<C>: FromEncodedPoint<C> + ToEncodedPoint<C>,
    FieldBytesSize<C>: ModulusSize,
{
    let n = FieldBytesSize::<C>::USIZE;
    let private = unsafe { std::slice::from_raw_parts(private, n) };
    let key = SigningKey::<C>::from_bytes(FieldBytes::<C>::from_slice(private))
        .map_err(|_| Error::user(Code::InvalidArgument))?;
    let key = key.verifying_key().as_affine().to_encoded_point(false);
    let public = unsafe { std::slice::from_raw_parts_mut(public, 2 * n) };
    let (x, y) = public.split_at_mut(n);
    x.copy_from_slice(key.x().ok_or(Error::user(0))?);
    y.copy_from_slice(key.y().ok_or(Error::user(0))?);
    Ok(())
}

#[unsafe(no_mangle)]
unsafe extern "C" fn env_cdi(params: api::sign::Params) -> isize {
    let api::sign::Params { curve, private, digest, r, s } = params;
    let res = match api::Curve::from(curve) {
        api::Curve::P256 => sign::<p256::NistP256, sha2::Sha256>(private, digest, r, s),
        api::Curve::P384 => sign::<p384::NistP384, sha2::Sha384>(private, digest, r, s),
    };
    convert_unit(res)
}

fn sign<C, D>(private: *const u8, digest: *const u8, r: *mut u8, s: *mut u8) -> Result<(), Error>
where
    C: PrimeCurve + CurveArithmetic,
    Scalar<C>: SignPrimitive<C>,
    SignatureSize<C>: ArrayLength<u8>,
    AffinePoint<C>: FromEncodedPoint<C> + ToEncodedPoint<C>,
    FieldBytesSize<C>: ModulusSize,
    D: Digest + BlockSizeUser + FixedOutput<OutputSize = FieldBytesSize<C>> + FixedOutputReset,
{
    let n = FieldBytesSize::<C>::USIZE;
    let private = unsafe { std::slice::from_raw_parts(private, n) };
    let key = SigningKey::<C>::from_bytes(FieldBytes::<C>::from_slice(private))
        .map_err(|_| Error::user(Code::InvalidArgument))?;
    let digest = unsafe { std::slice::from_raw_parts(digest, n) };
    let (sig, _) = key
        .as_nonzero_scalar()
        .try_sign_prehashed_rfc6979::<D>(FieldBytes::<C>::from_slice(digest), &[])
        .map_err(|_| Error::world(0))?;
    let r = unsafe { std::slice::from_raw_parts_mut(r, n) };
    let s = unsafe { std::slice::from_raw_parts_mut(s, n) };
    r.copy_from_slice(FieldBytes::<C>::from(sig.r()).as_slice());
    s.copy_from_slice(FieldBytes::<C>::from(sig.s()).as_slice());
    Ok(())
}

#[unsafe(no_mangle)]
unsafe extern "C" fn env_cdv(params: api::verify::Params) -> isize {
    let api::verify::Params { curve, public, digest, r, s } = params;
    let res = match api::Curve::from(curve) {
        api::Curve::P256 => verify::<p256::NistP256>(public, digest, r, s),
        api::Curve::P384 => verify::<p384::NistP384>(public, digest, r, s),
    };
    convert_bool(res)
}

fn verify<C>(
    public: *const u8, digest: *const u8, r: *const u8, s: *const u8,
) -> Result<bool, Error>
where
    C: PrimeCurve + CurveArithmetic,
    Scalar<C>: SignPrimitive<C>,
    SignatureSize<C>: ArrayLength<u8>,
    AffinePoint<C>: FromEncodedPoint<C> + ToEncodedPoint<C> + VerifyPrimitive<C>,
    FieldBytesSize<C>: ModulusSize,
{
    let n = FieldBytesSize::<C>::USIZE;
    let public = unsafe { std::slice::from_raw_parts(public, 2 * n) };
    let (x, y) = public.split_at(n);
    let key = EncodedPoint::<C>::from_affine_coordinates(x.into(), y.into(), false);
    let key = VerifyingKey::<C>::from_encoded_point(&key).map_err(|_| Error::user(0))?;
    let r = unsafe { std::slice::from_raw_parts(r, n) };
    let s = unsafe { std::slice::from_raw_parts(s, n) };
    let r = FieldBytes::<C>::from_slice(r).clone();
    let s = FieldBytes::<C>::from_slice(s).clone();
    let sig = Signature::from_scalars(r, s).map_err(|_| Error::user(0))?;
    let digest = unsafe { std::slice::from_raw_parts(digest, n) };
    Ok(key.verify_prehash(digest, &sig).is_ok())
}

#[unsafe(no_mangle)]
unsafe extern "C" fn env_cdd(params: api::drop::Params) -> isize {
    let api::drop::Params { curve, private } = params;
    let n = match api::Curve::from(curve) {
        api::Curve::P256 => 32,
        api::Curve::P384 => 48,
    };
    let private = unsafe { std::slice::from_raw_parts_mut(private, n) };
    private.zeroize();
    convert_unit(Ok(()))
}

#[unsafe(no_mangle)]
unsafe extern "C" fn env_cdw(params: api::wrap::Params) -> isize {
    let api::wrap::Params { curve, private, wrapped } = params;
    let n = match api::Curve::from(curve) {
        api::Curve::P256 => 32,
        api::Curve::P384 => 48,
    };
    let private = unsafe { std::slice::from_raw_parts(private, n) };
    let wrapped = unsafe { std::slice::from_raw_parts_mut(wrapped, n) };
    wrapped.copy_from_slice(private);
    convert_unit(Ok(()))
}

#[unsafe(no_mangle)]
unsafe extern "C" fn env_cdu(params: api::unwrap::Params) -> isize {
    let api::unwrap::Params { curve, wrapped, private } = params;
    let n = match api::Curve::from(curve) {
        api::Curve::P256 => 32,
        api::Curve::P384 => 48,
    };
    let private = unsafe { std::slice::from_raw_parts_mut(private, n) };
    let wrapped = unsafe { std::slice::from_raw_parts(wrapped, n) };
    private.copy_from_slice(wrapped);
    convert_unit(Ok(()))
}

#[unsafe(no_mangle)]
unsafe extern "C" fn env_cde(params: api::export::Params) -> isize {
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
unsafe extern "C" fn env_cdm(params: api::import::Params) -> isize {
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
