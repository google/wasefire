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

use wasefire_applet_api::crypto as crypto_api;
use wasefire_applet_api::crypto::hash::{self as api, Algorithm, Api};
use wasefire_board_api::crypto::hmac_sha256::{self, Api as _};
use wasefire_board_api::crypto::hmac_sha384::{self, Api as _};
use wasefire_board_api::crypto::sha256::Api as _;
use wasefire_board_api::crypto::sha384::Api as _;
use wasefire_board_api::crypto::Api as _;
use wasefire_board_api::Api as Board;

use crate::stores::HashContext;
use crate::{DispatchSchedulerCall, SchedulerCall, Trap};

pub fn process<B: Board>(call: Api<DispatchSchedulerCall<B>>) {
    match call {
        Api::IsSupported(call) => is_supported(call),
        Api::Initialize(call) => initialize(call),
        Api::Update(call) => update(call),
        Api::Finalize(call) => finalize(call),
        Api::IsHmacSupported(call) => is_hmac_supported(call),
        Api::HmacInitialize(call) => hmac_initialize(call),
        Api::HmacUpdate(call) => hmac_update(call),
        Api::HmacFinalize(call) => hmac_finalize(call),
        Api::IsHkdfSupported(call) => is_hkdf_supported(call),
        Api::HkdfExpand(call) => hkdf_expand(call),
    }
}

fn is_supported<B: Board>(mut call: SchedulerCall<B, api::is_supported::Sig>) {
    let api::is_supported::Params { algorithm } = call.read();
    let results = try {
        let supported = match Algorithm::try_from(*algorithm).map_err(|_| Trap)? {
            Algorithm::Sha256 => call.scheduler().board.crypto().sha256().is_supported(),
            Algorithm::Sha384 => call.scheduler().board.crypto().sha384().is_supported(),
        };
        api::is_supported::Results { supported: (supported as u32).into() }
    };
    call.reply(results)
}

fn initialize<B: Board>(mut call: SchedulerCall<B, api::initialize::Sig>) {
    let api::initialize::Params { algorithm } = call.read();
    let scheduler = call.scheduler();
    let results = try {
        let context = match Algorithm::try_from(*algorithm).map_err(|_| Trap)? {
            Algorithm::Sha256 => HashContext::Sha256(
                scheduler.board.crypto().sha256().initialize().map_err(|_| Trap)?,
            ),
            Algorithm::Sha384 => HashContext::Sha384(
                scheduler.board.crypto().sha384().initialize().map_err(|_| Trap)?,
            ),
        };
        let id = scheduler.applet.hashes.insert(context)? as u32;
        api::initialize::Results { id: id.into() }
    };
    call.reply(results);
}

fn update<B: Board>(mut call: SchedulerCall<B, api::update::Sig>) {
    let api::update::Params { id, data, length } = call.read();
    let scheduler = call.scheduler();
    let memory = scheduler.applet.store.memory();
    let results = try {
        let data = memory.get(*data, *length)?;
        match scheduler.applet.hashes.get_mut(*id as usize)? {
            HashContext::Sha256(context) => {
                scheduler.board.crypto().sha256().update(context, data).map_err(|_| Trap)?;
            }
            HashContext::Sha384(context) => {
                scheduler.board.crypto().sha384().update(context, data).map_err(|_| Trap)?;
            }
            _ => Err(Trap)?,
        }
        api::update::Results {}
    };
    call.reply(results);
}

fn finalize<B: Board>(mut call: SchedulerCall<B, api::finalize::Sig>) {
    let api::finalize::Params { id, digest } = call.read();
    let scheduler = call.scheduler();
    let memory = scheduler.applet.store.memory();
    let results = try {
        let context = scheduler.applet.hashes.take(*id as usize)?;
        let res = match context {
            _ if *digest == 0 => Ok(()),
            HashContext::Sha256(context) => {
                let digest = memory.get_array_mut::<32>(*digest)?;
                scheduler.board.crypto().sha256().finalize(context, digest)
            }
            HashContext::Sha384(context) => {
                let digest = memory.get_array_mut::<48>(*digest)?;
                scheduler.board.crypto().sha384().finalize(context, digest)
            }
            _ => Err(Trap)?,
        };
        let res = match res {
            Ok(()) => 0.into(),
            Err(_) => crypto_api::Error::InvalidArgument.into(),
        };
        api::finalize::Results { res }
    };
    call.reply(results);
}

fn is_hmac_supported<B: Board>(mut call: SchedulerCall<B, api::is_hmac_supported::Sig>) {
    let api::is_hmac_supported::Params { algorithm } = call.read();
    let results = try {
        let supported = match Algorithm::try_from(*algorithm).map_err(|_| Trap)? {
            Algorithm::Sha256 => call.scheduler().board.crypto().sha256().is_supported(),
            Algorithm::Sha384 => call.scheduler().board.crypto().sha384().is_supported(),
        };
        api::is_hmac_supported::Results { supported: (supported as u32).into() }
    };
    call.reply(results)
}

fn hmac_initialize<B: Board>(mut call: SchedulerCall<B, api::hmac_initialize::Sig>) {
    let api::hmac_initialize::Params { algorithm, key, key_len } = call.read();
    let scheduler = call.scheduler();
    let memory = scheduler.applet.store.memory();
    let results = try {
        let key = memory.get(*key, *key_len)?;
        let context = match Algorithm::try_from(*algorithm).map_err(|_| Trap)? {
            Algorithm::Sha256 => HashContext::HmacSha256(
                scheduler.board.crypto().hmac_sha256().initialize(key).map_err(|_| Trap)?,
            ),
            Algorithm::Sha384 => HashContext::HmacSha384(
                scheduler.board.crypto().hmac_sha384().initialize(key).map_err(|_| Trap)?,
            ),
        };
        let id = scheduler.applet.hashes.insert(context)? as u32;
        api::hmac_initialize::Results { id: id.into() }
    };
    call.reply(results);
}

fn hmac_update<B: Board>(mut call: SchedulerCall<B, api::hmac_update::Sig>) {
    let api::hmac_update::Params { id, data, length } = call.read();
    let scheduler = call.scheduler();
    let memory = scheduler.applet.store.memory();
    let results = try {
        let data = memory.get(*data, *length)?;
        match scheduler.applet.hashes.get_mut(*id as usize)? {
            HashContext::HmacSha256(context) => {
                scheduler.board.crypto().hmac_sha256().update(context, data).map_err(|_| Trap)?;
            }
            HashContext::HmacSha384(context) => {
                scheduler.board.crypto().hmac_sha384().update(context, data).map_err(|_| Trap)?;
            }
            _ => Err(Trap)?,
        }
        api::hmac_update::Results {}
    };
    call.reply(results);
}

fn hmac_finalize<B: Board>(mut call: SchedulerCall<B, api::hmac_finalize::Sig>) {
    let api::hmac_finalize::Params { id, hmac } = call.read();
    let scheduler = call.scheduler();
    let memory = scheduler.applet.store.memory();
    let results = try {
        let context = scheduler.applet.hashes.take(*id as usize)?;
        let res = match context {
            _ if *hmac == 0 => Ok(()),
            HashContext::HmacSha256(context) => {
                let hmac = memory.get_array_mut::<32>(*hmac)?;
                scheduler.board.crypto().hmac_sha256().finalize(context, hmac)
            }
            HashContext::HmacSha384(context) => {
                let hmac = memory.get_array_mut::<48>(*hmac)?;
                scheduler.board.crypto().hmac_sha384().finalize(context, hmac)
            }
            _ => Err(Trap)?,
        };
        let res = match res {
            Ok(()) => 0.into(),
            Err(_) => crypto_api::Error::InvalidArgument.into(),
        };
        api::hmac_finalize::Results { res }
    };
    call.reply(results);
}

fn is_hkdf_supported<B: Board>(mut call: SchedulerCall<B, api::is_hkdf_supported::Sig>) {
    let api::is_hkdf_supported::Params { algorithm } = call.read();
    let results = try {
        let supported = match Algorithm::try_from(*algorithm).map_err(|_| Trap)? {
            Algorithm::Sha256 => call.scheduler().board.crypto().sha256().is_supported(),
            Algorithm::Sha384 => call.scheduler().board.crypto().sha384().is_supported(),
        };
        api::is_hkdf_supported::Results { supported: (supported as u32).into() }
    };
    call.reply(results)
}

fn hkdf_expand<B: Board>(mut call: SchedulerCall<B, api::hkdf_expand::Sig>) {
    let api::hkdf_expand::Params { algorithm, prk, prk_len, info, info_len, okm, okm_len } =
        call.read();
    let scheduler = call.scheduler();
    let memory = scheduler.applet.store.memory();
    let results = try {
        let prk = memory.get(*prk, *prk_len)?;
        let info = memory.get(*info, *info_len)?;
        let okm = memory.get_mut(*okm, *okm_len)?;
        let res = match Algorithm::try_from(*algorithm).map_err(|_| Trap)? {
            Algorithm::Sha256 => {
                hkdf_sha256(scheduler.board.crypto().hmac_sha256(), prk, info, okm)
            }
            Algorithm::Sha384 => {
                hkdf_sha384(scheduler.board.crypto().hmac_sha384(), prk, info, okm)
            }
        };
        let res = match res {
            Ok(()) => 0.into(),
            Err(e) => e.into(),
        };
        api::hkdf_expand::Results { res }
    };
    call.reply(results);
}

fn hkdf_sha256<T: hmac_sha256::Types>(
    mut board: impl hmac_sha256::Api<T>, prk: &[u8], info: &[u8], okm: &mut [u8],
) -> Result<(), crypto_api::Error> {
    if 255 * 32 < okm.len() {
        return Err(crypto_api::Error::InvalidArgument);
    }
    let mut output = [0; 32];
    for (chunk, i) in okm.chunks_mut(32).zip(1u8 ..) {
        let mut hmac = board.initialize(prk).map_err(|_| crypto_api::Error::InvalidArgument)?;
        if 1 < i {
            board.update(&mut hmac, &output).map_err(|_| crypto_api::Error::InvalidArgument)?;
        }
        board.update(&mut hmac, info).map_err(|_| crypto_api::Error::InvalidArgument)?;
        board.update(&mut hmac, &[i]).map_err(|_| crypto_api::Error::InvalidArgument)?;
        board.finalize(hmac, &mut output).map_err(|_| crypto_api::Error::InvalidArgument)?;
        chunk.copy_from_slice(&output[.. chunk.len()]);
    }
    Ok(())
}

// TODO(#164): Merge this with hkdf_sha256 into a unified hkdf function.
fn hkdf_sha384<T: hmac_sha384::Types>(
    mut board: impl hmac_sha384::Api<T>, prk: &[u8], info: &[u8], okm: &mut [u8],
) -> Result<(), crypto_api::Error> {
    if 255 * 48 < okm.len() {
        return Err(crypto_api::Error::InvalidArgument);
    }
    let mut output = [0; 48];
    for (chunk, i) in okm.chunks_mut(48).zip(1u8 ..) {
        let mut hmac = board.initialize(prk).map_err(|_| crypto_api::Error::InvalidArgument)?;
        if 1 < i {
            board.update(&mut hmac, &output).map_err(|_| crypto_api::Error::InvalidArgument)?;
        }
        board.update(&mut hmac, info).map_err(|_| crypto_api::Error::InvalidArgument)?;
        board.update(&mut hmac, &[i]).map_err(|_| crypto_api::Error::InvalidArgument)?;
        board.finalize(hmac, &mut output).map_err(|_| crypto_api::Error::InvalidArgument)?;
        chunk.copy_from_slice(&output[.. chunk.len()]);
    }
    Ok(())
}
