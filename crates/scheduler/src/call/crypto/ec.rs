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

use wasefire_applet_api::crypto::ec::{self as api, Api, Curve};
use wasefire_board_api::crypto::p256::Api as _;
use wasefire_board_api::crypto::p384::Api as _;
use wasefire_board_api::crypto::Api as _;
use wasefire_board_api::Api as Board;

use crate::{DispatchSchedulerCall, SchedulerCall, Trap};

pub fn process<B: Board>(call: Api<DispatchSchedulerCall<B>>) {
    match call {
        Api::IsSupported(call) => is_supported(call),
        Api::IsValidScalar(call) => is_valid_scalar(call),
        Api::IsValidPoint(call) => is_valid_point(call),
        Api::BasePointMul(call) => base_point_mul(call),
        Api::PointMul(call) => point_mul(call),
        Api::EcdsaSign(call) => ecdsa_sign(call),
        Api::EcdsaVerify(call) => ecdsa_verify(call),
    }
}

fn is_supported<B: Board>(mut call: SchedulerCall<B, api::is_supported::Sig>) {
    let api::is_supported::Params { curve } = call.read();
    let results = try {
        let support = match convert_curve(*curve)? {
            Curve::P256 => call.scheduler().board.crypto().p256().is_supported(),
            Curve::P384 => call.scheduler().board.crypto().p384().is_supported(),
        };
        api::is_supported::Results { support: (support as u32).into() }
    };
    call.reply(results)
}

fn is_valid_scalar<B: Board>(mut call: SchedulerCall<B, api::is_valid_scalar::Sig>) {
    let api::is_valid_scalar::Params { curve, n } = call.read();
    let scheduler = call.scheduler();
    let memory = scheduler.applet.memory();
    let results = try {
        let valid = match convert_curve(*curve)? {
            Curve::P256 => {
                let n = memory.get_array::<32>(*n)?;
                scheduler.board.crypto().p256().is_valid_scalar(n)
            }
            Curve::P384 => {
                let n = memory.get_array::<48>(*n)?;
                scheduler.board.crypto().p384().is_valid_scalar(n)
            }
        };
        api::is_valid_scalar::Results { valid: (valid as u32).into() }
    };
    call.reply(results)
}

fn is_valid_point<B: Board>(mut call: SchedulerCall<B, api::is_valid_point::Sig>) {
    let api::is_valid_point::Params { curve, x, y } = call.read();
    let scheduler = call.scheduler();
    let memory = scheduler.applet.memory();
    let results = try {
        let valid = match convert_curve(*curve)? {
            Curve::P256 => {
                let x = memory.get_array::<32>(*x)?;
                let y = memory.get_array::<32>(*y)?;
                scheduler.board.crypto().p256().is_valid_point(x, y)
            }
            Curve::P384 => {
                let x = memory.get_array::<48>(*x)?;
                let y = memory.get_array::<48>(*y)?;
                scheduler.board.crypto().p384().is_valid_point(x, y)
            }
        };
        api::is_valid_point::Results { valid: (valid as u32).into() }
    };
    call.reply(results)
}

fn base_point_mul<B: Board>(mut call: SchedulerCall<B, api::base_point_mul::Sig>) {
    let api::base_point_mul::Params { curve, n, x, y } = call.read();
    let scheduler = call.scheduler();
    let memory = scheduler.applet.memory();
    let results = try {
        let res = match convert_curve(*curve)? {
            Curve::P256 => {
                let n = memory.get_array::<32>(*n)?;
                let x = memory.get_array_mut::<32>(*x)?;
                let y = memory.get_array_mut::<32>(*y)?;
                match scheduler.board.crypto().p256().base_point_mul(n, x, y) {
                    Ok(()) => 0u32,
                    Err(_) => u32::MAX,
                }
            }
            Curve::P384 => {
                let n = memory.get_array::<48>(*n)?;
                let x = memory.get_array_mut::<48>(*x)?;
                let y = memory.get_array_mut::<48>(*y)?;
                match scheduler.board.crypto().p384().base_point_mul(n, x, y) {
                    Ok(()) => 0u32,
                    Err(_) => u32::MAX,
                }
            }
        };
        api::base_point_mul::Results { res: res.into() }
    };
    call.reply(results);
}

fn point_mul<B: Board>(mut call: SchedulerCall<B, api::point_mul::Sig>) {
    let api::point_mul::Params { curve, n, in_x, in_y, out_x, out_y } = call.read();
    let scheduler = call.scheduler();
    let memory = scheduler.applet.memory();
    let results = try {
        let res = match convert_curve(*curve)? {
            Curve::P256 => {
                let n = memory.get_array::<32>(*n)?;
                let in_x = memory.get_array::<32>(*in_x)?;
                let in_y = memory.get_array::<32>(*in_y)?;
                let out_x = memory.get_array_mut::<32>(*out_x)?;
                let out_y = memory.get_array_mut::<32>(*out_y)?;
                match scheduler.board.crypto().p256().point_mul(n, in_x, in_y, out_x, out_y) {
                    Ok(()) => 0u32,
                    Err(_) => u32::MAX,
                }
            }
            Curve::P384 => {
                let n = memory.get_array::<48>(*n)?;
                let in_x = memory.get_array::<48>(*in_x)?;
                let in_y = memory.get_array::<48>(*in_y)?;
                let out_x = memory.get_array_mut::<48>(*out_x)?;
                let out_y = memory.get_array_mut::<48>(*out_y)?;
                match scheduler.board.crypto().p384().point_mul(n, in_x, in_y, out_x, out_y) {
                    Ok(()) => 0u32,
                    Err(_) => u32::MAX,
                }
            }
        };
        api::point_mul::Results { res: res.into() }
    };
    call.reply(results);
}

fn ecdsa_sign<B: Board>(mut call: SchedulerCall<B, api::ecdsa_sign::Sig>) {
    let api::ecdsa_sign::Params { curve, key, message, r, s } = call.read();
    let scheduler = call.scheduler();
    let memory = scheduler.applet.memory();
    let results = try {
        let res = match convert_curve(*curve)? {
            Curve::P256 => {
                let key = memory.get_array::<32>(*key)?;
                let message = memory.get_array::<32>(*message)?;
                let r = memory.get_array_mut::<32>(*r)?;
                let s = memory.get_array_mut::<32>(*s)?;
                match scheduler.board.crypto().p256().ecdsa_sign(key, message, r, s) {
                    Ok(()) => 0u32,
                    Err(_) => u32::MAX,
                }
            }
            Curve::P384 => {
                let key = memory.get_array::<48>(*key)?;
                let message = memory.get_array::<48>(*message)?;
                let r = memory.get_array_mut::<48>(*r)?;
                let s = memory.get_array_mut::<48>(*s)?;
                match scheduler.board.crypto().p384().ecdsa_sign(key, message, r, s) {
                    Ok(()) => 0u32,
                    Err(_) => u32::MAX,
                }
            }
        };
        api::ecdsa_sign::Results { res: res.into() }
    };
    call.reply(results);
}

fn ecdsa_verify<B: Board>(mut call: SchedulerCall<B, api::ecdsa_verify::Sig>) {
    let api::ecdsa_verify::Params { curve, message, x, y, r, s } = call.read();
    let scheduler = call.scheduler();
    let memory = scheduler.applet.memory();
    let results = try {
        let res = match convert_curve(*curve)? {
            Curve::P256 => {
                let message = memory.get_array::<32>(*message)?;
                let x = memory.get_array::<32>(*x)?;
                let y = memory.get_array::<32>(*y)?;
                let r = memory.get_array::<32>(*r)?;
                let s = memory.get_array::<32>(*s)?;
                match scheduler.board.crypto().p256().ecdsa_verify(message, x, y, r, s) {
                    Ok(true) => 1u32,
                    Ok(false) => 0u32,
                    Err(_) => u32::MAX,
                }
            }
            Curve::P384 => {
                let message = memory.get_array::<48>(*message)?;
                let x = memory.get_array::<48>(*x)?;
                let y = memory.get_array::<48>(*y)?;
                let r = memory.get_array::<48>(*r)?;
                let s = memory.get_array::<48>(*s)?;
                match scheduler.board.crypto().p384().ecdsa_verify(message, x, y, r, s) {
                    Ok(true) => 1u32,
                    Ok(false) => 0u32,
                    Err(_) => u32::MAX,
                }
            }
        };
        api::ecdsa_verify::Results { res: res.into() }
    };
    call.reply(results);
}

fn convert_curve(x: u32) -> Result<Curve, Trap> {
    Curve::try_from(x).map_err(|_| Trap)
}
