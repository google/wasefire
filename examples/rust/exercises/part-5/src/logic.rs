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

use alloc::string::String;

use interface::{Request, Response};
use wasefire::crypto;

use crate::store::Store;
use crate::touch::Touch;

pub struct Logic {
    store: Store,
    touch: Touch, // NEW
}

impl Logic {
    pub fn new() -> Self {
        let store = Store::new();
        let touch = Touch::new(); // NEW
        Logic { store, touch }
    }

    pub fn process(&mut self, request: Request) -> Result<Response, String> {
        self.touch.consume()?; // NEW
        Ok(match request {
            Request::Register { name } => {
                let private = Private::random().map_err(|_| "generation failed")?;
                let public = private.public_key();
                self.store.insert(name, (*private.private_key()).into())?;
                Response::Register { x: (*public.x()).into(), y: (*public.y()).into() }
            }
            Request::Authenticate { name, challenge } => {
                let private = self.store.find(&name)?;
                let private = Private::from_non_zero_scalar(private.into()).unwrap();
                let signature =
                    private.sign_prehash((&challenge).into()).map_err(|_| "signature failed")?;
                Response::Authenticate { r: (*signature.r()).into(), s: (*signature.s()).into() }
            }
            Request::List => Response::List { names: self.store.list() },
            Request::Delete { name } => {
                self.store.delete(&name)?;
                Response::Delete
            }
        })
    }
}

type Private = crypto::ec::EcdsaPrivate<crypto::ec::P256>;
