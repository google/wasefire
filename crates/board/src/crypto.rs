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
pub mod sha256;

/// Returns this [`Types`] given a [`crate::Types`].
pub type Get<B> = <super::Get<B> as super::Types>::Crypto;

/// Associated types of [`Api`].
pub trait Types {
    type Sha256: sha256::Types;
}

/// Cryptography interface.
pub trait Api<T: Types> {
    // TODO: Rename to Aes128Ccm.
    type Ccm<'a>: ccm::Api
    where Self: 'a;
    fn ccm(&mut self) -> Self::Ccm<'_>;

    // TODO: Rename to Aes256Gcm.
    type Gcm<'a>: gcm::Api
    where Self: 'a;
    fn gcm(&mut self) -> Self::Gcm<'_>;

    type Sha256<'a>: sha256::Api<T::Sha256>
    where Self: 'a;
    fn sha256(&mut self) -> Self::Sha256<'_>;
}

impl Types for Unimplemented {
    type Sha256 = Unimplemented;
}

impl Api<Unimplemented> for Unimplemented {
    type Ccm<'a> = Unimplemented;
    fn ccm(&mut self) -> Self::Ccm<'_> {
        unreachable!()
    }

    type Gcm<'a> = Unimplemented;
    fn gcm(&mut self) -> Self::Gcm<'_> {
        unreachable!()
    }

    type Sha256<'a> = Unimplemented;
    fn sha256(&mut self) -> Self::Sha256<'_> {
        unreachable!()
    }
}

impl Types for Unsupported {
    type Sha256 = Unsupported;
}

impl Api<Unsupported> for Unsupported {
    type Ccm<'a> = Unsupported;
    fn ccm(&mut self) -> Self::Ccm<'_> {
        Unsupported
    }

    type Gcm<'a> = Unsupported;
    fn gcm(&mut self) -> Self::Gcm<'_> {
        Unsupported
    }

    type Sha256<'a> = Unsupported;
    fn sha256(&mut self) -> Self::Sha256<'_> {
        Unsupported
    }
}
