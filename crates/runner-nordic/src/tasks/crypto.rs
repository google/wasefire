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

use wasefire_board_api::crypto::{Api, Types};
use wasefire_board_api::Unsupported;

use crate::tasks::Board;

mod ccm;

impl Types for Board {
    type HmacSha256 = Unsupported;
    type HmacSha384 = Unsupported;
    type Sha256 = Unsupported;
    type Sha384 = Unsupported;
}

impl Api<Board> for &mut Board {
    type Aes128Ccm<'a> = &'a mut Board where Self: 'a;
    fn aes128_ccm(&mut self) -> &mut Board {
        self
    }

    type Aes256Gcm<'a> = Unsupported where Self: 'a;
    fn aes256_gcm(&mut self) -> Unsupported {
        Unsupported
    }

    type HmacSha256<'a> = Unsupported where Self: 'a;
    fn hmac_sha256(&mut self) -> Unsupported {
        Unsupported
    }

    type HmacSha384<'a> = Unsupported where Self: 'a;
    fn hmac_sha384(&mut self) -> Unsupported {
        Unsupported
    }

    type P256<'a> = Unsupported where Self: 'a;
    fn p256(&mut self) -> Unsupported {
        Unsupported
    }

    type P384<'a> = Unsupported where Self: 'a;
    fn p384(&mut self) -> Unsupported {
        Unsupported
    }

    type Sha256<'a> = Unsupported where Self: 'a;
    fn sha256(&mut self) -> Unsupported {
        Unsupported
    }

    type Sha384<'a> = Unsupported where Self: 'a;
    fn sha384(&mut self) -> Unsupported {
        Unsupported
    }
}
