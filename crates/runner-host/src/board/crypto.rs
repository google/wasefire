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

use crate::board::Board;

mod ccm;
mod gcm;

impl Types for Board {
    type Sha256 = Unsupported;
}

impl Api<Board> for &mut Board {
    type Ccm<'a> = &'a mut Board
    where Self: 'a;
    fn ccm(&mut self) -> Self::Ccm<'_> {
        self
    }

    type Gcm<'a> = &'a mut Board
    where Self: 'a;
    fn gcm(&mut self) -> Self::Gcm<'_> {
        self
    }

    type Sha256<'a> = Unsupported
    where Self: 'a;
    fn sha256(&mut self) -> Unsupported {
        Unsupported
    }
}
