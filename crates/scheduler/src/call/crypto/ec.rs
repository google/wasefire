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
        Api::BasePointMul(call) => base_point_mul(call),
        Api::PointMul(call) => point_mul(call),
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

fn convert_curve(x: u32) -> Result<Curve, Trap> {
    Curve::try_from(x).map_err(|_| Trap)
}
