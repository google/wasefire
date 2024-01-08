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

pub(crate) fn new() -> Item {
    let docs = docs!();
    let name = "scheduling".into();
    let items = vec![
        item! {
            /// Waits until a callback is scheduled.
            ///
            /// This can be used as power management, since the CPU will sleep while waiting.
            ///
            /// Returns zero on success.
            fn wait_for_callback "sw" {}
        },
        item! {
            /// Returns how many callbacks are pending.
            fn num_pending_callbacks "sh" {}
        },
        item! {
            /// Aborts the applet.
            ///
            /// Does not return on success.
            fn abort "sa" {}
        },
    ];
    Item::Mod(Mod { docs, name, items })
}
