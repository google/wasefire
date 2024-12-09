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

#[cfg(feature = "board-api-crypto-p384")]
use generic_array as _;
use wasefire_applet_api::crypto::ec::{self as api, Api, Curve};
use wasefire_board_api::Api as Board;
#[cfg(feature = "internal-board-api-crypto-ecc")]
use wasefire_board_api::crypto::ecc::Api as _;
#[cfg(feature = "internal-board-api-crypto-ecc")]
use wasefire_board_api::{self as board, Support};

#[cfg(feature = "internal-board-api-crypto-ecc")]
use crate::applet::store::MemoryApi;
use crate::{DispatchSchedulerCall, SchedulerCall, Trap};

pub fn process<B: Board>(call: Api<DispatchSchedulerCall<B>>) {
    match call {
        Api::IsSupported(call) => is_supported(call),
        Api::IsValidScalar(call) => {
            or_fail!("internal-board-api-crypto-ecc", is_valid_scalar(call))
        }
        Api::IsValidPoint(call) => or_fail!("internal-board-api-crypto-ecc", is_valid_point(call)),
        Api::BasePointMul(call) => or_fail!("internal-board-api-crypto-ecc", base_point_mul(call)),
        Api::PointMul(call) => or_fail!("internal-board-api-crypto-ecc", point_mul(call)),
        Api::EcdsaSign(call) => or_fail!("internal-board-api-crypto-ecc", ecdsa_sign(call)),
        Api::EcdsaVerify(call) => or_fail!("internal-board-api-crypto-ecc", ecdsa_verify(call)),
    }
}

fn is_supported<B: Board>(call: SchedulerCall<B, api::is_supported::Sig>) {
    let api::is_supported::Params { curve } = call.read();
    call.reply(try { convert_curve::<B>(*curve)?.is_ok() })
}

#[cfg(feature = "internal-board-api-crypto-ecc")]
fn is_valid_scalar<B: Board>(mut call: SchedulerCall<B, api::is_valid_scalar::Sig>) {
    let api::is_valid_scalar::Params { curve, n } = call.read();
    let applet = call.applet();
    let memory = applet.memory();
    let result = try {
        match convert_curve::<B>(*curve)?? {
            #[cfg(feature = "board-api-crypto-p256")]
            Curve::P256 => {
                let n = memory.get_array::<32>(*n)?.into();
                board::crypto::P256::<B>::is_valid_scalar(n)
            }
            #[cfg(feature = "board-api-crypto-p384")]
            Curve::P384 => {
                let n = memory.get_array::<48>(*n)?.into();
                board::crypto::P384::<B>::is_valid_scalar(n)
            }
            #[allow(unreachable_patterns)]
            _ => trap_use!(),
        }
    };
    call.reply(result)
}

#[cfg(feature = "internal-board-api-crypto-ecc")]
fn is_valid_point<B: Board>(mut call: SchedulerCall<B, api::is_valid_point::Sig>) {
    let api::is_valid_point::Params { curve, x, y } = call.read();
    let applet = call.applet();
    let memory = applet.memory();
    let result = try {
        match convert_curve::<B>(*curve)?? {
            #[cfg(feature = "board-api-crypto-p256")]
            Curve::P256 => {
                let x = memory.get_array::<32>(*x)?.into();
                let y = memory.get_array::<32>(*y)?.into();
                board::crypto::P256::<B>::is_valid_point(x, y)
            }
            #[cfg(feature = "board-api-crypto-p384")]
            Curve::P384 => {
                let x = memory.get_array::<48>(*x)?.into();
                let y = memory.get_array::<48>(*y)?.into();
                board::crypto::P384::<B>::is_valid_point(x, y)
            }
            #[allow(unreachable_patterns)]
            _ => trap_use!(),
        }
    };
    call.reply(result)
}

#[cfg(feature = "internal-board-api-crypto-ecc")]
fn base_point_mul<B: Board>(mut call: SchedulerCall<B, api::base_point_mul::Sig>) {
    let api::base_point_mul::Params { curve, n, x, y } = call.read();
    let applet = call.applet();
    let memory = applet.memory();
    let result = try {
        match convert_curve::<B>(*curve)?? {
            #[cfg(feature = "board-api-crypto-p256")]
            Curve::P256 => {
                let n = memory.get_array::<32>(*n)?.into();
                let x = memory.get_array_mut::<32>(*x)?.into();
                let y = memory.get_array_mut::<32>(*y)?.into();
                board::crypto::P256::<B>::base_point_mul(n, x, y)?
            }
            #[cfg(feature = "board-api-crypto-p384")]
            Curve::P384 => {
                let n = memory.get_array::<48>(*n)?.into();
                let x = memory.get_array_mut::<48>(*x)?.into();
                let y = memory.get_array_mut::<48>(*y)?.into();
                board::crypto::P384::<B>::base_point_mul(n, x, y)?
            }
            #[allow(unreachable_patterns)]
            _ => trap_use!(),
        }
    };
    call.reply(result);
}

#[cfg(feature = "internal-board-api-crypto-ecc")]
fn point_mul<B: Board>(mut call: SchedulerCall<B, api::point_mul::Sig>) {
    let api::point_mul::Params { curve, n, in_x, in_y, out_x, out_y } = call.read();
    let applet = call.applet();
    let memory = applet.memory();
    let result = try {
        match convert_curve::<B>(*curve)?? {
            #[cfg(feature = "board-api-crypto-p256")]
            Curve::P256 => {
                let n = memory.get_array::<32>(*n)?.into();
                let in_x = memory.get_array::<32>(*in_x)?.into();
                let in_y = memory.get_array::<32>(*in_y)?.into();
                let out_x = memory.get_array_mut::<32>(*out_x)?.into();
                let out_y = memory.get_array_mut::<32>(*out_y)?.into();
                board::crypto::P256::<B>::point_mul(n, in_x, in_y, out_x, out_y)?
            }
            #[cfg(feature = "board-api-crypto-p384")]
            Curve::P384 => {
                let n = memory.get_array::<48>(*n)?.into();
                let in_x = memory.get_array::<48>(*in_x)?.into();
                let in_y = memory.get_array::<48>(*in_y)?.into();
                let out_x = memory.get_array_mut::<48>(*out_x)?.into();
                let out_y = memory.get_array_mut::<48>(*out_y)?.into();
                board::crypto::P384::<B>::point_mul(n, in_x, in_y, out_x, out_y)?
            }
            #[allow(unreachable_patterns)]
            _ => trap_use!(),
        }
    };
    call.reply(result);
}

#[cfg(feature = "internal-board-api-crypto-ecc")]
fn ecdsa_sign<B: Board>(mut call: SchedulerCall<B, api::ecdsa_sign::Sig>) {
    let api::ecdsa_sign::Params { curve, key, message, r, s } = call.read();
    let applet = call.applet();
    let memory = applet.memory();
    let result = try {
        match convert_curve::<B>(*curve)?? {
            #[cfg(feature = "board-api-crypto-p256")]
            Curve::P256 => {
                let key = memory.get_array::<32>(*key)?.into();
                let message = memory.get_array::<32>(*message)?.into();
                let r = memory.get_array_mut::<32>(*r)?.into();
                let s = memory.get_array_mut::<32>(*s)?.into();
                board::crypto::P256::<B>::ecdsa_sign(key, message, r, s)?
            }
            #[cfg(feature = "board-api-crypto-p384")]
            Curve::P384 => {
                let key = memory.get_array::<48>(*key)?.into();
                let message = memory.get_array::<48>(*message)?.into();
                let r = memory.get_array_mut::<48>(*r)?.into();
                let s = memory.get_array_mut::<48>(*s)?.into();
                board::crypto::P384::<B>::ecdsa_sign(key, message, r, s)?
            }
            #[allow(unreachable_patterns)]
            _ => trap_use!(),
        }
    };
    call.reply(result);
}

#[cfg(feature = "internal-board-api-crypto-ecc")]
fn ecdsa_verify<B: Board>(mut call: SchedulerCall<B, api::ecdsa_verify::Sig>) {
    let api::ecdsa_verify::Params { curve, message, x, y, r, s } = call.read();
    let applet = call.applet();
    let memory = applet.memory();
    let result = try {
        match convert_curve::<B>(*curve)?? {
            #[cfg(feature = "board-api-crypto-p256")]
            Curve::P256 => {
                let message = memory.get_array::<32>(*message)?.into();
                let x = memory.get_array::<32>(*x)?.into();
                let y = memory.get_array::<32>(*y)?.into();
                let r = memory.get_array::<32>(*r)?.into();
                let s = memory.get_array::<32>(*s)?.into();
                board::crypto::P256::<B>::ecdsa_verify(message, x, y, r, s)?
            }
            #[cfg(feature = "board-api-crypto-p384")]
            Curve::P384 => {
                let message = memory.get_array::<48>(*message)?.into();
                let x = memory.get_array::<48>(*x)?.into();
                let y = memory.get_array::<48>(*y)?.into();
                let r = memory.get_array::<48>(*r)?.into();
                let s = memory.get_array::<48>(*s)?.into();
                board::crypto::P384::<B>::ecdsa_verify(message, x, y, r, s)?
            }
            #[allow(unreachable_patterns)]
            _ => trap_use!(),
        }
    };
    call.reply(result);
}

#[allow(clippy::extra_unused_type_parameters)]
fn convert_curve<B: Board>(curve: u32) -> Result<Result<Curve, Trap>, Trap> {
    let curve = Curve::try_from(curve).map_err(|_| Trap)?;
    let support = match curve {
        Curve::P256 => or_false!("board-api-crypto-p256", board::crypto::P256::<B>::SUPPORT),
        Curve::P384 => or_false!("board-api-crypto-p384", board::crypto::P384::<B>::SUPPORT),
    };
    Ok(support.then_some(curve).ok_or(Trap))
}
