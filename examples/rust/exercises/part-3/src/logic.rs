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

use alloc::collections::btree_map::Entry;
use alloc::collections::BTreeMap;
use alloc::string::String;

use interface::{Request, Response};
use wasefire::crypto;

pub struct Logic {
    /// Maps from name to private key.
    ///
    /// This will use the device persistent storage in a later exercise.
    store: BTreeMap<String, [u8; 32]>,
}

impl Logic {
    pub fn new() -> Self {
        todo!()
    }

    pub fn process(&mut self, request: Request) -> Result<Response, String> {
        Ok(match request {
            Request::Register { name } => {
                // TODO: Use Private::random() to generate a private key.
                // TODO: Use private.public_key() to get the public key.
                // TODO: Insert private.private_key() in the store. Fail if already present.
                // TODO: Use public.x() and public.y() to build the response.
                Response::Register { x: todo!(), y: todo!() }
            }
            Request::Authenticate { name, challenge } => {
                // TODO: Get the private key from the store. Fail if absent.
                // TODO: Use Private::from_non_zero_scalar() to convert it.
                // TODO: Use private.sign_prehash() to sign the challenge.
                // TODO: Use signature.r() and signature.s() to build the response.
                Response::Authenticate { r: todo!(), s: todo!() }
            }
            Request::List => Response::List { names: todo!() },
            Request::Delete { name } => {
                // TODO: Remove the name from the store. Fail if absent.
                todo!();
                Response::Delete
            }
        })
    }
}

type Private = crypto::ec::EcdsaPrivate<crypto::ec::P256>;
