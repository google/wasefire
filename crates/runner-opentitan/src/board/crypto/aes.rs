// Copyright 2025 Google LLC
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

use typenum::{U16, U32};
use wasefire_board_api::Supported;
use wasefire_board_api::crypto::cbc::{Api, Array};
use wasefire_error::Error;

use crate::crypto::aes;

pub enum Impl {}

impl Supported for Impl {}

impl Api<U32, U16> for Impl {
    fn encrypt(key: &Array<U32>, iv: &Array<U16>, blocks: &mut [u8]) -> Result<(), Error> {
        aes::encrypt_cbc(key.as_array().unwrap(), iv.as_array().unwrap(), blocks)
    }

    fn decrypt(key: &Array<U32>, iv: &Array<U16>, blocks: &mut [u8]) -> Result<(), Error> {
        aes::decrypt_cbc(key.as_array().unwrap(), iv.as_array().unwrap(), blocks)
    }
}
