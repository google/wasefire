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

#![allow(unused_imports)]

use std::ptr::hash;

use digest::{FixedOutput, InvalidLength, KeyInit, Output, Update};
use generic_array::GenericArray;
use wasefire_applet_api::crypto::hash::{self as api, Algorithm, Api};
use wasefire_applet_api::crypto::Api::Hash;
use wasefire_board_api::crypto::{HashApi, HmacApi, LastError};
use wasefire_board_api::{self as board, Api as Board, Support};
use wasefire_error::Error;

use crate::applet::store::{MemoryApi, StoreApi};
#[cfg(feature = "internal-hash-context")]
use crate::applet::HashContext;
use crate::{DispatchSchedulerCall, SchedulerCall, Trap};

pub fn process<B: Board>(call: Api<DispatchSchedulerCall<B>>) {
    match call {
        #[cfg(feature = "applet-api-crypto-hash")]
        Api::IsSupported(call) => is_supported(call),
        #[cfg(feature = "applet-api-crypto-hash")]
        Api::Initialize(call) => initialize(call),
        #[cfg(feature = "applet-api-crypto-hash")]
        Api::Update(call) => update(call),
        #[cfg(feature = "applet-api-crypto-hash")]
        Api::Finalize(call) => finalize(call),
        #[cfg(feature = "applet-api-crypto-hmac")]
        Api::IsHmacSupported(call) => is_hmac_supported(call),
        #[cfg(feature = "applet-api-crypto-hmac")]
        Api::HmacInitialize(call) => hmac_initialize(call),
        #[cfg(feature = "applet-api-crypto-hmac")]
        Api::HmacUpdate(call) => hmac_update(call),
        #[cfg(feature = "applet-api-crypto-hmac")]
        Api::HmacFinalize(call) => hmac_finalize(call),
        #[cfg(feature = "applet-api-crypto-hkdf")]
        Api::IsHkdfSupported(call) => is_hkdf_supported(call),
        #[cfg(feature = "applet-api-crypto-hkdf")]
        Api::HkdfExpand(call) => hkdf_expand(call),
    }
}

#[cfg(feature = "applet-api-crypto-hash")]
fn is_supported<B: Board>(call: SchedulerCall<B, api::is_supported::Sig>) {
    let api::is_supported::Params { algorithm } = call.read();
    call.reply(convert_hash_algorithm::<B>(*algorithm).map(|x| Ok(x.is_ok())))
}

#[cfg(feature = "applet-api-crypto-hash")]
fn initialize<B: Board>(mut call: SchedulerCall<B, api::initialize::Sig>) {
    let api::initialize::Params { algorithm } = call.read();
    let scheduler = call.scheduler();
    let result = try {
        let context_or_error = match convert_hash_algorithm::<B>(*algorithm)?? {
            #[cfg(feature = "board-api-crypto-sha256")]
            Algorithm::Sha256 => {
                let hash_or_error = HashApi::<board::crypto::Sha256<B>>::new();
                match hash_or_error {
                    Ok(hash) => Ok(HashContext::Sha256(hash)),
                    Err(error) => Err(error),
                }
            }
            #[cfg(feature = "board-api-crypto-sha384")]
            Algorithm::Sha384 => {
                let hash_or_error = HashApi::<board::crypto::Sha384<B>>::new();
                match hash_or_error {
                    Ok(hash) => Ok(HashContext::Sha384(hash)),
                    Err(error) => Err(error),
                }
            }
            #[allow(unreachable_patterns)]
            _ => Err(Trap)?,
        };
        match context_or_error {
            Ok(context) => Ok(scheduler.applet.hashes.insert(context)? as u32),
            Err(error) => Err(error),
        }
    };
    call.reply(result);
}

#[cfg(feature = "applet-api-crypto-hash")]
fn update<B: Board>(mut call: SchedulerCall<B, api::update::Sig>) {
    let api::update::Params { id, data, length } = call.read();
    let scheduler = call.scheduler();
    let memory = scheduler.applet.store.memory();
    let result = try {
        let data = memory.get(*data, *length)?;
        match scheduler.applet.hashes.get_mut(*id as usize)? {
            #[cfg(feature = "board-api-crypto-sha256")]
            HashContext::Sha256(context) => {
                context.update(data);
                context.last_error()
            }
            #[cfg(feature = "board-api-crypto-sha384")]
            HashContext::Sha384(context) => {
                context.update(data);
                context.last_error()
            }
            _ => trap_use!(data),
        }
    };
    call.reply(result);
}

#[cfg(feature = "applet-api-crypto-hash")]
fn finalize<B: Board>(mut call: SchedulerCall<B, api::finalize::Sig>) {
    let api::finalize::Params { id, digest } = call.read();
    let scheduler = call.scheduler();
    let memory = scheduler.applet.store.memory();
    let result = try {
        let context = scheduler.applet.hashes.take(*id as usize)?;
        match context {
            _ if *digest == 0 => (),
            #[cfg(feature = "board-api-crypto-sha256")]
            HashContext::Sha256(context) => {
                let digest = memory.get_array_mut::<32>(*digest)?;
                context.finalize_into(GenericArray::from_mut_slice(digest));
                context.last_error()
            }
            #[cfg(feature = "board-api-crypto-sha384")]
            HashContext::Sha384(context) => {
                let digest = memory.get_array_mut::<48>(*digest)?;
                context.finalize_into(GenericArray::from_mut_slice(digest));
                context.last_error()
            }
            _ => trap_use!(memory),
        };
    };
    call.reply(result);
}

#[cfg(feature = "applet-api-crypto-hmac")]
fn is_hmac_supported<B: Board>(call: SchedulerCall<B, api::is_hmac_supported::Sig>) {
    let api::is_hmac_supported::Params { algorithm } = call.read();
    call.reply(convert_hmac_algorithm::<B>(*algorithm).map(|x| Ok(x.is_ok())))
}

#[cfg(feature = "applet-api-crypto-hmac")]
fn hmac_initialize<B: Board>(mut call: SchedulerCall<B, api::hmac_initialize::Sig>) {
    let api::hmac_initialize::Params { algorithm, key, key_len } = call.read();
    let scheduler = call.scheduler();
    let memory = scheduler.applet.memory();
    let result = try {
        let key = memory.get(*key, *key_len)?;
        let context_or_error = match convert_hmac_algorithm::<B>(*algorithm)?? {
            #[cfg(feature = "board-api-crypto-hmac-sha256")]
            Algorithm::Sha256 => {
                let hmac_or_error = HmacApi::<board::crypto::HmacSha256<B>>::new(key);
                match hmac_or_error {
                    Ok(hmac) => Ok(HashContext::HmacSha256(hmac)),
                    Err(error) => Err(error),
                }
            }
            #[cfg(feature = "board-api-crypto-hmac-sha384")]
            Algorithm::Sha384 => {
                let hmac_or_error = HmacApi::<board::crypto::HmacSha384<B>>::new(key);
                match hmac_or_error {
                    Ok(hmac) => Ok(HashContext::HmacSha384(hmac)),
                    Err(error) => Err(error),
                }
            }
            #[allow(unreachable_patterns)]
            _ => trap_use!(key),
        };
        match context_or_error {
            Ok(context) => Ok(scheduler.applet.hashes.insert(context)? as u32),
            Err(error) => Err(error),
        }
    };
    call.reply(result);
}

#[cfg(feature = "applet-api-crypto-hmac")]
fn hmac_update<B: Board>(mut call: SchedulerCall<B, api::hmac_update::Sig>) {
    let api::hmac_update::Params { id, data, length } = call.read();
    let scheduler = call.scheduler();
    let memory = scheduler.applet.store.memory();
    let result = try {
        let data = memory.get(*data, *length)?;
        match scheduler.applet.hashes.get_mut(*id as usize)? {
            #[cfg(feature = "board-api-crypto-hmac-sha256")]
            HashContext::HmacSha256(context) => {
                context.update(data);
                context.last_error()
            }
            #[cfg(feature = "board-api-crypto-hmac-sha384")]
            HashContext::HmacSha384(context) => {
                context.update(data);
                context.last_error()
            }
            _ => trap_use!(data),
        }
    };
    call.reply(result);
}

#[cfg(feature = "applet-api-crypto-hmac")]
fn hmac_finalize<B: Board>(mut call: SchedulerCall<B, api::hmac_finalize::Sig>) {
    let api::hmac_finalize::Params { id, hmac } = call.read();
    let scheduler = call.scheduler();
    let memory = scheduler.applet.store.memory();
    let result = try {
        let context = scheduler.applet.hashes.take(*id as usize)?;
        match context {
            _ if *hmac == 0 => (),
            #[cfg(feature = "board-api-crypto-hmac-sha256")]
            HashContext::HmacSha256(context) => {
                let hmac = memory.get_array_mut::<32>(*hmac)?;
                context.finalize_into(GenericArray::from_mut_slice(hmac));
                context.last_error()
            }
            #[cfg(feature = "board-api-crypto-hmac-sha384")]
            HashContext::HmacSha384(context) => {
                let hmac = memory.get_array_mut::<48>(*hmac)?;
                context.finalize_into(GenericArray::from_mut_slice(hmac));
                context.last_error()
            }
            _ => trap_use!(memory),
        }
    };
    call.reply(result);
}

#[cfg(feature = "applet-api-crypto-hkdf")]
fn is_hkdf_supported<B: Board>(call: SchedulerCall<B, api::is_hkdf_supported::Sig>) {
    let api::is_hkdf_supported::Params { algorithm } = call.read();
    call.reply(convert_hmac_algorithm::<B>(*algorithm).map(|x| Ok(x.is_ok())))
}

#[cfg(feature = "applet-api-crypto-hkdf")]
fn hkdf_expand<B: Board>(mut call: SchedulerCall<B, api::hkdf_expand::Sig>) {
    let api::hkdf_expand::Params { algorithm, prk, prk_len, info, info_len, okm, okm_len } =
        call.read();
    let scheduler = call.scheduler();
    let memory = scheduler.applet.memory();
    let result = try {
        let prk = memory.get(*prk, *prk_len)?;
        let info = memory.get(*info, *info_len)?;
        let okm = memory.get_mut(*okm, *okm_len)?;
        let res: Result<(), _> = match convert_hmac_algorithm::<B>(*algorithm)?? {
            #[cfg(feature = "board-api-crypto-hmac-sha256")]
            Algorithm::Sha256 => hkdf::<board::crypto::HmacSha256<B>>(prk, info, okm),
            #[cfg(feature = "board-api-crypto-hmac-sha384")]
            Algorithm::Sha384 => hkdf::<board::crypto::HmacSha384<B>>(prk, info, okm),
            #[allow(unreachable_patterns)]
            _ => trap_use!(prk, info, okm),
        };
        res.map_err(|InvalidLength| Error::user(0))
    };
    call.reply(result);
}

// TODO(https://github.com/RustCrypto/KDFs/issues/80): We should ideally use the hkdf crate.
#[allow(dead_code)]
#[cfg(feature = "applet-api-crypto-hkdf")]
fn hkdf<H: KeyInit + Update + FixedOutput>(
    prk: &[u8], info: &[u8], okm: &mut [u8],
) -> Result<(), InvalidLength> {
    if 255 * H::output_size() < okm.len() {
        return Err(InvalidLength);
    }
    let mut output = Output::<H>::default();
    for (chunk, i) in okm.chunks_mut(H::output_size()).zip(1u8 ..) {
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

#[allow(clippy::extra_unused_type_parameters)]
#[cfg(feature = "applet-api-crypto-hash")]
fn convert_hash_algorithm<B: Board>(algorithm: u32) -> Result<Result<Algorithm, Trap>, Trap> {
    let algorithm = Algorithm::try_from(algorithm).map_err(|_| Trap)?;
    let support = match algorithm {
        Algorithm::Sha256 => {
            or_false!("board-api-crypto-sha256", board::crypto::Sha256::<B>::SUPPORT)
        }
        Algorithm::Sha384 => {
            or_false!("board-api-crypto-sha384", board::crypto::Sha384::<B>::SUPPORT)
        }
    };
    Ok(support.then_some(algorithm).ok_or(Trap))
}

#[allow(clippy::extra_unused_type_parameters)]
#[cfg(any(feature = "applet-api-crypto-hmac", feature = "applet-api-crypto-hkdf"))]
fn convert_hmac_algorithm<B: Board>(algorithm: u32) -> Result<Result<Algorithm, Trap>, Trap> {
    let algorithm = Algorithm::try_from(algorithm).map_err(|_| Trap)?;
    let support = match algorithm {
        Algorithm::Sha256 => {
            or_false!("board-api-crypto-hmac-sha256", board::crypto::HmacSha256::<B>::SUPPORT)
        }
        Algorithm::Sha384 => {
            or_false!("board-api-crypto-hmac-sha384", board::crypto::HmacSha384::<B>::SUPPORT)
        }
    };
    Ok(support.then_some(algorithm).ok_or(Trap))
}
