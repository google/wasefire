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

//! Cryptography interface.

use crate::{Unimplemented, Unsupported};

pub mod ccm;
pub mod gcm;

/// Cryptography interface.
pub trait Api {
    type Ccm<'a>: ccm::Api
    where Self: 'a;
    fn ccm(&mut self) -> Self::Ccm<'_>;

    type Gcm<'a>: gcm::Api
    where Self: 'a;
    fn gcm(&mut self) -> Self::Gcm<'_>;
}

impl Api for Unimplemented {
    type Ccm<'a> = Unimplemented;
    fn ccm(&mut self) -> Self::Ccm<'_> {
        unreachable!()
    }

    type Gcm<'a> = Unimplemented;
    fn gcm(&mut self) -> Self::Gcm<'_> {
        unreachable!()
    }
}

impl Api for Unsupported {
    type Ccm<'a> = Unsupported;
    fn ccm(&mut self) -> Self::Ccm<'_> {
        Unsupported
    }

    type Gcm<'a> = Unsupported;
    fn gcm(&mut self) -> Self::Gcm<'_> {
        Unsupported
    }
}
