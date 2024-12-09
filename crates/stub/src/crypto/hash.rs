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

use std::sync::Mutex;

use crypto_common::{InvalidLength, KeyInit, OutputSizeUser};
use digest::{FixedOutput, Update};
use hkdf::{Hkdf, HmacImpl};
use hmac::Hmac;
use sha2::{Sha256, Sha384};
use wasefire_applet_api::crypto::hash as api;
use wasefire_error::Error;
use wasefire_logger as log;

use crate::{convert, convert_unit};

enum Context {
    Sha256(Sha256),
    Sha384(Sha384),
    HmacSha256(Hmac<Sha256>),
    HmacSha384(Hmac<Sha384>),
}

struct Contexts(Vec<Option<Context>>);

static CONTEXTS: Mutex<Contexts> = Mutex::new(Contexts(Vec::new()));

impl Contexts {
    fn insert(&mut self, context: Context) -> usize {
        for (id, slot) in self.0.iter_mut().enumerate() {
            if slot.is_none() {
                *slot = Some(context);
                return id;
            }
        }
        let id = self.0.len();
        self.0.push(Some(context));
        id
    }

    fn get(&mut self, id: usize) -> &mut Context {
        self.0[id].as_mut().unwrap()
    }

    fn take(&mut self, id: usize) -> Context {
        self.0[id].take().unwrap()
    }
}

#[unsafe(no_mangle)]
unsafe extern "C" fn env_chs(_: api::is_supported::Params) -> isize {
    1
}

#[unsafe(no_mangle)]
unsafe extern "C" fn env_chi(params: api::initialize::Params) -> isize {
    let api::initialize::Params { algorithm } = params;
    let context = match api::Algorithm::from(algorithm) {
        api::Algorithm::Sha256 => Context::Sha256(Sha256::default()),
        api::Algorithm::Sha384 => Context::Sha384(Sha384::default()),
    };
    convert(Ok(CONTEXTS.lock().unwrap().insert(context) as u32))
}

#[unsafe(no_mangle)]
unsafe extern "C" fn env_chu(params: api::update::Params) -> isize {
    let api::update::Params { id, data, length } = params;
    let data = unsafe { std::slice::from_raw_parts(data, length) };
    match CONTEXTS.lock().unwrap().get(id) {
        Context::Sha256(x) => x.update(data),
        Context::Sha384(x) => x.update(data),
        _ => log::panic!("Invalid context"),
    };
    0
}

#[unsafe(no_mangle)]
unsafe extern "C" fn env_chf(params: api::finalize::Params) -> isize {
    let api::finalize::Params { id, digest } = params;
    let digest = |length| unsafe { std::slice::from_raw_parts_mut(digest, length) };
    match CONTEXTS.lock().unwrap().take(id) {
        Context::Sha256(x) => x.finalize_into(digest(32).into()),
        Context::Sha384(x) => x.finalize_into(digest(48).into()),
        _ => log::panic!("Invalid context"),
    };
    0
}

#[unsafe(no_mangle)]
unsafe extern "C" fn env_cht(_: api::is_hmac_supported::Params) -> isize {
    1
}

#[unsafe(no_mangle)]
unsafe extern "C" fn env_chj(params: api::hmac_initialize::Params) -> isize {
    let api::hmac_initialize::Params { algorithm, key, key_len } = params;
    let key = unsafe { std::slice::from_raw_parts(key, key_len) };
    let context = try {
        match api::Algorithm::from(algorithm) {
            api::Algorithm::Sha256 => Context::HmacSha256(Hmac::new_from_slice(key)?),
            api::Algorithm::Sha384 => Context::HmacSha384(Hmac::new_from_slice(key)?),
        }
    };
    let res = match context {
        Ok(context) => Ok(CONTEXTS.lock().unwrap().insert(context) as u32),
        Err(InvalidLength) => Err(Error::user(0)),
    };
    convert(res)
}

#[unsafe(no_mangle)]
unsafe extern "C" fn env_chv(params: api::hmac_update::Params) -> isize {
    let api::hmac_update::Params { id, data, length } = params;
    let data = unsafe { std::slice::from_raw_parts(data, length) };
    match CONTEXTS.lock().unwrap().get(id) {
        Context::HmacSha256(x) => x.update(data),
        Context::HmacSha384(x) => x.update(data),
        _ => log::panic!("Invalid context"),
    };
    0
}

#[unsafe(no_mangle)]
unsafe extern "C" fn env_chg(params: api::hmac_finalize::Params) -> isize {
    let api::hmac_finalize::Params { id, hmac } = params;
    let hmac = |length| unsafe { std::slice::from_raw_parts_mut(hmac, length) };
    match CONTEXTS.lock().unwrap().take(id) {
        Context::HmacSha256(x) => x.finalize_into(hmac(32).into()),
        Context::HmacSha384(x) => x.finalize_into(hmac(48).into()),
        _ => log::panic!("Invalid context"),
    };
    0
}

#[unsafe(no_mangle)]
unsafe extern "C" fn env_chr(_: api::is_hkdf_supported::Params) -> isize {
    1
}

#[unsafe(no_mangle)]
unsafe extern "C" fn env_che(params: api::hkdf_expand::Params) -> isize {
    let api::hkdf_expand::Params { algorithm, prk, prk_len, info, info_len, okm, okm_len } = params;
    let prk = unsafe { std::slice::from_raw_parts(prk, prk_len) };
    let info = unsafe { std::slice::from_raw_parts(info, info_len) };
    let okm = unsafe { std::slice::from_raw_parts_mut(okm, okm_len) };
    let res = match api::Algorithm::from(algorithm) {
        api::Algorithm::Sha256 => hkdf::<Sha256, Hmac<Sha256>>(prk, info, okm),
        api::Algorithm::Sha384 => hkdf::<Sha384, Hmac<Sha384>>(prk, info, okm),
    };
    convert_unit(res)
}

fn hkdf<H: OutputSizeUser, I: HmacImpl<H>>(
    prk: &[u8], info: &[u8], okm: &mut [u8],
) -> Result<(), Error> {
    let hkdf = Hkdf::<H, I>::from_prk(prk).map_err(|_| Error::user(0))?;
    hkdf.expand(info, okm).map_err(|_| Error::user(0))
}
