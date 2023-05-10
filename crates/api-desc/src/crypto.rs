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

use crate::*;

mod ccm;
mod ec;
mod gcm;
mod hash;

pub(crate) fn new() -> Item {
    let docs = docs! {
        /// Cryptographic operations.
    };
    let name = "crypto".into();
    let items = vec![
        item! {
            /// Describes errors on cryptographic operations.
            enum Error {
                /// A function pre-condition was broken.
                InvalidArgument = 0,

                /// An operation is unsupported.
                Unsupported = 1,
            }
        },
        ccm::new(),
        ec::new(),
        gcm::new(),
        hash::new(),
    ];
    Item::Mod(Mod { docs, name, items })
}
