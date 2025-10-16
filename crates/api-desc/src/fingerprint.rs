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

use crate::*;

#[cfg(feature = "api-fingerprint-matcher")]
mod matcher;
#[cfg(feature = "api-fingerprint-sensor")]
mod sensor;

pub(crate) fn new() -> Item {
    let docs = docs! {
        /// Fingerprint operations.
    };
    let name = "fingerprint".into();
    let items = vec![
        #[cfg(feature = "api-fingerprint-matcher")]
        matcher::new(),
        #[cfg(feature = "api-fingerprint-sensor")]
        sensor::new(),
        item! {
            /// Register a handler for finger detection.
            fn register "fr" {
                /// Function called on finger detection.
                ///
                /// The function takes its opaque `data` as argument.
                handler_func: fn { data: *const void },

                /// The opaque data to use when calling the handler function.
                handler_data: *const void,
            } -> ()
        },
        item! {
            /// Unregister handlers for finger detection.
            fn unregister "fu" {} -> ()
        },
    ];
    Item::Mod(Mod { docs, name, items })
}
