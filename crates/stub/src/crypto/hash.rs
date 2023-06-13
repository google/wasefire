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

use sha2::Digest;
use wasefire_applet_api::crypto::hash as api;

enum Context {
    Sha256(sha2::Sha256),
    Sha384(sha2::Sha384),
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
        api::Algorithm::Sha256 => Context::Sha256(sha2::Sha256::default()),
        api::Algorithm::Sha384 => Context::Sha384(sha2::Sha384::default()),
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
    };
}

#[no_mangle]
unsafe extern "C" fn chf(params: api::finalize::Params) -> api::finalize::Results {
    let api::finalize::Params { id, digest } = params;
    let data = |length| unsafe { std::slice::from_raw_parts_mut(digest, length) };
    match CONTEXTS.lock().unwrap().take(id) {
        Context::Sha256(x) => x.finalize_into(data(32).into()),
        Context::Sha384(x) => x.finalize_into(data(48).into()),
    };
    api::finalize::Results { res: 0 }
}
