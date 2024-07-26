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

use crypto_common::generic_array::{ArrayLength, GenericArray};
use crypto_common::BlockSizeUser;
use digest::{Digest, FixedOutput, FixedOutputReset};
use ecdsa::hazmat::{SignPrimitive, VerifyPrimitive};
use ecdsa::{Signature, SignatureSize, VerifyingKey};
use elliptic_curve::ops::MulByGenerator;
use elliptic_curve::sec1::{EncodedPoint, FromEncodedPoint, ModulusSize, ToEncodedPoint};
use elliptic_curve::subtle::CtOption;
use elliptic_curve::{
    AffinePoint, CurveArithmetic, FieldBytesSize, PrimeCurve, ProjectivePoint, Scalar,
    ScalarPrimitive,
};
use signature::hazmat::PrehashVerifier;
use wasefire_applet_api::crypto::ec as api;
use wasefire_error::Error;

use crate::{convert_bool, convert_unit};

#[no_mangle]
unsafe extern "C" fn env_ces(_: api::is_supported::Params) -> isize {
    1
}

#[no_mangle]
unsafe extern "C" fn env_cet(params: api::is_valid_scalar::Params) -> isize {
    let api::is_valid_scalar::Params { curve, n } = params;
    let valid = match api::Curve::from(curve) {
        api::Curve::P256 => unsafe { is_valid_scalar::<p256::NistP256>(n) },
        api::Curve::P384 => unsafe { is_valid_scalar::<p384::NistP384>(n) },
    };
    convert_bool(Ok(valid))
}

#[no_mangle]
unsafe extern "C" fn env_ceq(params: api::is_valid_point::Params) -> isize {
    let api::is_valid_point::Params { curve, x, y } = params;
    let valid = match api::Curve::from(curve) {
        api::Curve::P256 => unsafe { is_valid_point::<p256::NistP256>(x, y) },
        api::Curve::P384 => unsafe { is_valid_point::<p384::NistP384>(x, y) },
    };
    convert_bool(Ok(valid))
}

#[no_mangle]
unsafe extern "C" fn env_ceb(params: api::base_point_mul::Params) -> isize {
    let api::base_point_mul::Params { curve, n, x, y } = params;
    let res = match api::Curve::from(curve) {
        api::Curve::P256 => unsafe { base_point_mul::<p256::NistP256>(n, x, y) },
        api::Curve::P384 => unsafe { base_point_mul::<p384::NistP384>(n, x, y) },
    };
    convert_unit(res)
}

#[no_mangle]
unsafe extern "C" fn env_cep(params: api::point_mul::Params) -> isize {
    let api::point_mul::Params { curve, n, in_x, in_y, out_x, out_y } = params;
    let res = match api::Curve::from(curve) {
        api::Curve::P256 => unsafe { point_mul::<p256::NistP256>(n, in_x, in_y, out_x, out_y) },
        api::Curve::P384 => unsafe { point_mul::<p384::NistP384>(n, in_x, in_y, out_x, out_y) },
    };
    convert_unit(res)
}

#[no_mangle]
unsafe extern "C" fn env_cei(params: api::ecdsa_sign::Params) -> isize {
    let api::ecdsa_sign::Params { curve, key, message, r, s } = params;
    let res = match api::Curve::from(curve) {
        api::Curve::P256 => unsafe {
            ecdsa_sign::<p256::NistP256, sha2::Sha256>(key, message, r, s)
        },
        api::Curve::P384 => unsafe {
            ecdsa_sign::<p384::NistP384, sha2::Sha384>(key, message, r, s)
        },
    };
    convert_unit(res)
}

#[no_mangle]
unsafe extern "C" fn env_cev(params: api::ecdsa_verify::Params) -> isize {
    let api::ecdsa_verify::Params { curve, message, x, y, r, s } = params;
    let res = match api::Curve::from(curve) {
        api::Curve::P256 => unsafe { ecdsa_verify::<p256::NistP256>(message, x, y, r, s) },
        api::Curve::P384 => unsafe { ecdsa_verify::<p384::NistP384>(message, x, y, r, s) },
    };
    convert_bool(res)
}

unsafe fn array_from_ptr<'a, N: ArrayLength<u8>>(n: *const u8) -> &'a GenericArray<u8, N> {
    GenericArray::from_slice(unsafe { std::slice::from_raw_parts(n, N::to_usize()) })
}

unsafe fn array_from_ptr_mut<'a, N: ArrayLength<u8>>(n: *mut u8) -> &'a mut GenericArray<u8, N> {
    GenericArray::from_mut_slice(unsafe { std::slice::from_raw_parts_mut(n, N::to_usize()) })
}

unsafe fn is_valid_scalar<C: CurveArithmetic>(n: *const u8) -> bool {
    scalar_from_int::<C>(unsafe { array_from_ptr(n) }).is_ok()
}

unsafe fn is_valid_point<C>(x: *const u8, y: *const u8) -> bool
where
    C: CurveArithmetic,
    ProjectivePoint<C>: FromEncodedPoint<C>,
    FieldBytesSize<C>: ModulusSize,
{
    point_from_ints::<C>(unsafe { array_from_ptr(x) }, unsafe { array_from_ptr(y) }).is_ok()
}

unsafe fn base_point_mul<C>(n: *const u8, x: *mut u8, y: *mut u8) -> Result<(), Error>
where
    C: CurveArithmetic,
    AffinePoint<C>: ToEncodedPoint<C>,
    FieldBytesSize<C>: ModulusSize,
{
    let n = unsafe { array_from_ptr(n) };
    let x = unsafe { array_from_ptr_mut(x) };
    let y = unsafe { array_from_ptr_mut(y) };
    let r = ProjectivePoint::<C>::mul_by_generator(&scalar_from_int::<C>(n)?);
    point_to_ints::<C>(&r.into(), x, y)
}

unsafe fn point_mul<C>(
    n: *const u8, in_x: *const u8, in_y: *const u8, out_x: *mut u8, out_y: *mut u8,
) -> Result<(), Error>
where
    C: PrimeCurve + CurveArithmetic,
    AffinePoint<C>: ToEncodedPoint<C>,
    ProjectivePoint<C>: FromEncodedPoint<C>,
    FieldBytesSize<C>: ModulusSize,
{
    let n = unsafe { array_from_ptr(n) };
    let in_x = unsafe { array_from_ptr(in_x) };
    let in_y = unsafe { array_from_ptr(in_y) };
    let out_x = unsafe { array_from_ptr_mut(out_x) };
    let out_y = unsafe { array_from_ptr_mut(out_y) };
    let r = point_from_ints::<C>(in_x, in_y)? * scalar_from_int::<C>(n)?;
    point_to_ints::<C>(&r.into(), out_x, out_y)
}

unsafe fn ecdsa_sign<C, D>(d: *const u8, m: *const u8, r: *mut u8, s: *mut u8) -> Result<(), Error>
where
    C: PrimeCurve + CurveArithmetic,
    D: Digest + BlockSizeUser + FixedOutput<OutputSize = FieldBytesSize<C>> + FixedOutputReset,
    SignatureSize<C>: ArrayLength<u8>,
    Scalar<C>: SignPrimitive<C>,
{
    let d = unsafe { array_from_ptr(d) };
    let m = unsafe { array_from_ptr(m) };
    let r = unsafe { array_from_ptr_mut::<FieldBytesSize<C>>(r) };
    let s = unsafe { array_from_ptr_mut::<FieldBytesSize<C>>(s) };
    let d = scalar_from_int::<C>(d)?;
    let (signature, _) = d.try_sign_prehashed_rfc6979::<D>(m, &[]).unwrap();
    r.copy_from_slice(&scalar_to_int::<C>(signature.r()));
    s.copy_from_slice(&scalar_to_int::<C>(signature.s()));
    Ok(())
}

unsafe fn ecdsa_verify<C>(
    m: *const u8, x: *const u8, y: *const u8, r: *const u8, s: *const u8,
) -> Result<bool, Error>
where
    C: PrimeCurve + CurveArithmetic,
    AffinePoint<C>: FromEncodedPoint<C> + ToEncodedPoint<C> + VerifyPrimitive<C>,
    FieldBytesSize<C>: ModulusSize,
    SignatureSize<C>: ArrayLength<u8>,
{
    let m = unsafe { array_from_ptr::<FieldBytesSize<C>>(m) };
    let x = unsafe { array_from_ptr(x) };
    let y = unsafe { array_from_ptr(y) };
    let r = unsafe { array_from_ptr(r) };
    let s = unsafe { array_from_ptr(s) };
    let p = EncodedPoint::<C>::from_affine_coordinates(x, y, false);
    let p = VerifyingKey::<C>::from_encoded_point(&p).map_err(|_| Error::user(0))?;
    let signature = Signature::from_scalars(r.clone(), s.clone()).map_err(|_| Error::user(0))?;
    Ok(p.verify_prehash(m, &signature).is_ok())
}

type Int<C> = GenericArray<u8, FieldBytesSize<C>>;

fn scalar_from_int<C: CurveArithmetic>(n: &Int<C>) -> Result<Scalar<C>, Error> {
    Ok(convert_ct(ScalarPrimitive::from_bytes(n)).ok_or(Error::user(0))?.into())
}

fn scalar_to_int<C: CurveArithmetic>(n: impl AsRef<Scalar<C>>) -> Int<C> {
    (*n.as_ref()).into()
}

fn point_from_ints<C>(x: &Int<C>, y: &Int<C>) -> Result<ProjectivePoint<C>, Error>
where
    C: CurveArithmetic,
    ProjectivePoint<C>: FromEncodedPoint<C>,
    FieldBytesSize<C>: ModulusSize,
{
    let r = EncodedPoint::<C>::from_affine_coordinates(x, y, false);
    convert_ct(ProjectivePoint::<C>::from_encoded_point(&r)).ok_or(Error::user(0))
}

fn point_to_ints<C>(p: &AffinePoint<C>, x: &mut Int<C>, y: &mut Int<C>) -> Result<(), Error>
where
    C: CurveArithmetic,
    AffinePoint<C>: ToEncodedPoint<C>,
    FieldBytesSize<C>: ModulusSize,
{
    let p = p.to_encoded_point(false);
    x.copy_from_slice(p.x().ok_or(Error::user(0))?);
    y.copy_from_slice(p.y().ok_or(Error::user(0))?);
    Ok(())
}

fn convert_ct<T>(x: CtOption<T>) -> Option<T> {
    Option::<T>::from(x)
}
