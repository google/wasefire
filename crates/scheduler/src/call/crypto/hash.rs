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

use digest::{FixedOutput, InvalidLength, KeyInit, Output, Update};
use generic_array::GenericArray;
use wasefire_applet_api::crypto as crypto_api;
use wasefire_applet_api::crypto::hash::{self as api, Algorithm, Api};
use wasefire_board_api::{self as board, Api as Board, Support};

use crate::applet::store::{MemoryApi, StoreApi};
use crate::applet::HashContext;
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

fn is_supported<B: Board>(call: SchedulerCall<B, api::is_supported::Sig>) {
    let api::is_supported::Params { algorithm } = call.read();
    let results = try {
        let supported = convert_hash_algorithm::<B>(*algorithm)?.is_ok() as u32;
        api::is_supported::Results { supported: supported.into() }
    };
    call.reply(results)
}

fn initialize<B: Board>(mut call: SchedulerCall<B, api::initialize::Sig>) {
    let api::initialize::Params { algorithm } = call.read();
    let scheduler = call.scheduler();
    let results = try {
        let context = match convert_hash_algorithm::<B>(*algorithm)?? {
            Algorithm::Sha256 => HashContext::Sha256(board::crypto::Sha256::<B>::default()),
            Algorithm::Sha384 => HashContext::Sha384(board::crypto::Sha384::<B>::default()),
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
            HashContext::Sha256(context) => context.update(data),
            HashContext::Sha384(context) => context.update(data),
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
        match context {
            _ if *digest == 0 => (),
            HashContext::Sha256(context) => {
                let digest = memory.get_array_mut::<32>(*digest)?;
                context.finalize_into(GenericArray::from_mut_slice(digest))
            }
            HashContext::Sha384(context) => {
                let digest = memory.get_array_mut::<48>(*digest)?;
                context.finalize_into(GenericArray::from_mut_slice(digest))
            }
            _ => Err(Trap)?,
        }
        api::finalize::Results { res: 0.into() }
    };
    call.reply(results);
}

fn is_hmac_supported<B: Board>(call: SchedulerCall<B, api::is_hmac_supported::Sig>) {
    let api::is_hmac_supported::Params { algorithm } = call.read();
    let results = try {
        let supported = convert_hmac_algorithm::<B>(*algorithm)?.is_ok() as u32;
        api::is_hmac_supported::Results { supported: supported.into() }
    };
    call.reply(results)
}

fn hmac_initialize<B: Board>(mut call: SchedulerCall<B, api::hmac_initialize::Sig>) {
    let api::hmac_initialize::Params { algorithm, key, key_len } = call.read();
    let scheduler = call.scheduler();
    let memory = scheduler.applet.memory();
    let results = try {
        let key = memory.get(*key, *key_len)?;
        let context = match convert_hmac_algorithm::<B>(*algorithm)?? {
            Algorithm::Sha256 => HashContext::HmacSha256(
                board::crypto::HmacSha256::<B>::new_from_slice(key).map_err(|_| Trap)?,
            ),
            Algorithm::Sha384 => HashContext::HmacSha384(
                board::crypto::HmacSha384::<B>::new_from_slice(key).map_err(|_| Trap)?,
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
            HashContext::HmacSha256(context) => context.update(data),
            HashContext::HmacSha384(context) => context.update(data),
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
        match context {
            _ if *hmac == 0 => (),
            HashContext::HmacSha256(context) => {
                let hmac = memory.get_array_mut::<32>(*hmac)?;
                context.finalize_into(GenericArray::from_mut_slice(hmac))
            }
            HashContext::HmacSha384(context) => {
                let hmac = memory.get_array_mut::<48>(*hmac)?;
                context.finalize_into(GenericArray::from_mut_slice(hmac))
            }
            _ => Err(Trap)?,
        }
        api::hmac_finalize::Results { res: 0.into() }
    };
    call.reply(results);
}

fn is_hkdf_supported<B: Board>(call: SchedulerCall<B, api::is_hkdf_supported::Sig>) {
    let api::is_hkdf_supported::Params { algorithm } = call.read();
    let results = try {
        let supported = convert_hmac_algorithm::<B>(*algorithm)?.is_ok() as u32;
        api::is_hkdf_supported::Results { supported: supported.into() }
    };
    call.reply(results)
}

fn hkdf_expand<B: Board>(mut call: SchedulerCall<B, api::hkdf_expand::Sig>) {
    let api::hkdf_expand::Params { algorithm, prk, prk_len, info, info_len, okm, okm_len } =
        call.read();
    let scheduler = call.scheduler();
    let memory = scheduler.applet.memory();
    let results = try {
        let prk = memory.get(*prk, *prk_len)?;
        let info = memory.get(*info, *info_len)?;
        let okm = memory.get_mut(*okm, *okm_len)?;
        let res = match convert_hmac_algorithm::<B>(*algorithm)?? {
            Algorithm::Sha256 => hkdf::<board::crypto::HmacSha256<B>>(prk, info, okm),
            Algorithm::Sha384 => hkdf::<board::crypto::HmacSha384<B>>(prk, info, okm),
        };
        let res = match res {
            Ok(()) => 0.into(),
            Err(InvalidLength) => crypto_api::Error::InvalidArgument.into(),
        };
        api::hkdf_expand::Results { res }
    };
    call.reply(results);
}

// TODO(https://github.com/RustCrypto/KDFs/issues/80): We should ideally use the hkdf crate.
fn hkdf<H: KeyInit + Update + FixedOutput>(
    prk: &[u8], info: &[u8], okm: &mut [u8],
) -> Result<(), InvalidLength> {
    if 255 * H::output_size() < okm.len() {
        return Err(InvalidLength);
    }
    let mut output = Output::<H>::default();
    for (chunk, i) in okm.chunks_mut(32).zip(1u8 ..) {
        let mut hmac = <H as KeyInit>::new_from_slice(prk)?;
        if 1 < i {
            hmac.update(&output);
        }
        hmac.update(info);
        hmac.update(&[i]);
        hmac.finalize_into(&mut output);
        chunk.copy_from_slice(&output[.. chunk.len()]);
    }
    Ok(())
}

fn convert_hash_algorithm<B: Board>(algorithm: u32) -> Result<Result<Algorithm, Trap>, Trap> {
    let algorithm = Algorithm::try_from(algorithm).map_err(|_| Trap)?;
    let support = match algorithm {
        Algorithm::Sha256 => board::crypto::Sha256::<B>::SUPPORT,
        Algorithm::Sha384 => board::crypto::Sha384::<B>::SUPPORT,
    };
    Ok(support.then_some(algorithm).ok_or(Trap))
}

fn convert_hmac_algorithm<B: Board>(algorithm: u32) -> Result<Result<Algorithm, Trap>, Trap> {
    let algorithm = Algorithm::try_from(algorithm).map_err(|_| Trap)?;
    let support = match algorithm {
        Algorithm::Sha256 => board::crypto::HmacSha256::<B>::SUPPORT,
        Algorithm::Sha384 => board::crypto::HmacSha384::<B>::SUPPORT,
    };
    Ok(support.then_some(algorithm).ok_or(Trap))
}
