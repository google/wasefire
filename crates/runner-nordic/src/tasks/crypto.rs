// Copyright 2022 Google LLC
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

use wasefire_board_api::crypto::Api;
use wasefire_board_api::Unsupported;

mod ccm;

pub enum Impl {}

impl Api for Impl {
    type Aes128Ccm = ccm::Impl;
    type Aes256Gcm = Unsupported;
    type HmacSha256 = Unsupported;
    type HmacSha384 = Unsupported;
    type P256 = Unsupported;
    type P384 = Unsupported;
    type Sha256 = Unsupported;
    type Sha384 = Unsupported;
}
