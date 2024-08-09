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

#[cfg(feature = "api-usb-serial")]
mod serial;

#[cfg(feature = "api-usb-ctap")]
mod ctap;

pub(crate) fn new() -> Item {
    let docs = docs! {
        /// USB operations.
    };
    let name = "usb".into();
    let items = vec![
        #[cfg(feature = "api-usb-serial")]
        serial::new(),
        #[cfg(feature = "api-usb-ctap")]
        ctap::new(),
    ];
    Item::Mod(Mod { docs, name, items })
}
