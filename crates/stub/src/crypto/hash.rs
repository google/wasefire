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
use wasefire_applet_api::crypto::{hash as api, Error};
use wasefire_logger as log;

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

#[no_mangle]
unsafe extern "C" fn chs(_: api::is_supported::Params) -> api::is_supported::Results {
    api::is_supported::Results { supported: 1 }
}

#[no_mangle]
unsafe extern "C" fn chi(params: api::initialize::Params) -> api::initialize::Results {
    let api::initialize::Params { algorithm } = params;
    let context = match api::Algorithm::from(algorithm) {
        api::Algorithm::Sha256 => Context::Sha256(Sha256::default()),
        api::Algorithm::Sha384 => Context::Sha384(Sha384::default()),
    };
    let id = CONTEXTS.lock().unwrap().insert(context) as isize;
    api::initialize::Results { id }
}

#[no_mangle]
unsafe extern "C" fn chu(params: api::update::Params) {
    let api::update::Params { id, data, length } = params;
    let data = unsafe { std::slice::from_raw_parts(data, length) };
    match CONTEXTS.lock().unwrap().get(id) {
        Context::Sha256(x) => x.update(data),
        Context::Sha384(x) => x.update(data),
        _ => log::panic!("Invalid context"),
    };
}

#[no_mangle]
unsafe extern "C" fn chf(params: api::finalize::Params) -> api::finalize::Results {
    let api::finalize::Params { id, digest } = params;
    let digest = |length| unsafe { std::slice::from_raw_parts_mut(digest, length) };
    match CONTEXTS.lock().unwrap().take(id) {
        Context::Sha256(x) => x.finalize_into(digest(32).into()),
        Context::Sha384(x) => x.finalize_into(digest(48).into()),
        _ => log::panic!("Invalid context"),
    };
    api::finalize::Results { res: 0 }
}

#[no_mangle]
unsafe extern "C" fn cht(_: api::is_hmac_supported::Params) -> api::is_hmac_supported::Results {
    api::is_hmac_supported::Results { supported: 1 }
}

#[no_mangle]
unsafe extern "C" fn chj(params: api::hmac_initialize::Params) -> api::hmac_initialize::Results {
    let api::hmac_initialize::Params { algorithm, key, key_len } = params;
    let key = unsafe { std::slice::from_raw_parts(key, key_len) };
    let context: Result<Context, InvalidLength> = try {
        match api::Algorithm::from(algorithm) {
            api::Algorithm::Sha256 => Context::HmacSha256(Hmac::new_from_slice(key)?),
            api::Algorithm::Sha384 => Context::HmacSha384(Hmac::new_from_slice(key)?),
        }
    };
    let id = match context {
        Ok(context) => CONTEXTS.lock().unwrap().insert(context) as isize,
        Err(_) => Error::InvalidArgument.into(),
    };
    api::hmac_initialize::Results { id }
}

#[no_mangle]
unsafe extern "C" fn chv(params: api::hmac_update::Params) {
    let api::hmac_update::Params { id, data, length } = params;
    let data = unsafe { std::slice::from_raw_parts(data, length) };
    match CONTEXTS.lock().unwrap().get(id) {
        Context::HmacSha256(x) => x.update(data),
        Context::HmacSha384(x) => x.update(data),
        _ => log::panic!("Invalid context"),
    };
}

#[no_mangle]
unsafe extern "C" fn chg(params: api::hmac_finalize::Params) -> api::hmac_finalize::Results {
    let api::hmac_finalize::Params { id, hmac } = params;
    let hmac = |length| unsafe { std::slice::from_raw_parts_mut(hmac, length) };
    match CONTEXTS.lock().unwrap().take(id) {
        Context::HmacSha256(x) => x.finalize_into(hmac(32).into()),
        Context::HmacSha384(x) => x.finalize_into(hmac(48).into()),
        _ => log::panic!("Invalid context"),
    };
    api::hmac_finalize::Results { res: 0 }
}

#[no_mangle]
unsafe extern "C" fn chr(_: api::is_hkdf_supported::Params) -> api::is_hkdf_supported::Results {
    api::is_hkdf_supported::Results { supported: 1 }
}

#[no_mangle]
unsafe extern "C" fn che(params: api::hkdf_expand::Params) -> api::hkdf_expand::Results {
    let api::hkdf_expand::Params { algorithm, prk, prk_len, info, info_len, okm, okm_len } = params;
    let prk = unsafe { std::slice::from_raw_parts(prk, prk_len) };
    let info = unsafe { std::slice::from_raw_parts(info, info_len) };
    let okm = unsafe { std::slice::from_raw_parts_mut(okm, okm_len) };
    let res = match api::Algorithm::from(algorithm) {
        api::Algorithm::Sha256 => hkdf::<Sha256, Hmac<Sha256>>(prk, info, okm),
        api::Algorithm::Sha384 => hkdf::<Sha384, Hmac<Sha384>>(prk, info, okm),
    };
    let res = match res {
        Ok(()) => 0,
        Err(err) => err.into(),
    };
    api::hkdf_expand::Results { res }
}

fn hkdf<H: OutputSizeUser, I: HmacImpl<H>>(
    prk: &[u8], info: &[u8], okm: &mut [u8],
) -> Result<(), Error> {
    let hkdf = Hkdf::<H, I>::from_prk(prk).map_err(|_| Error::InvalidArgument)?;
    hkdf.expand(info, okm).map_err(|_| Error::InvalidArgument)
}
