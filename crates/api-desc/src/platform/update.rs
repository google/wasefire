// Copyright 2023 Google LLC
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
    let docs = docs! {
        /// Operations to update the platform.
        ///
        /// All operations are abstract over the update content such that they can work on all
        /// platforms. In particular, chunks and errors are platform-specific. Applets with
        /// knowledge about their platform may actually inspect that content for additional checks.
    };
    let name = "update".into();
    let items = vec![
        item! {
            /// Whether platform update is supported.
            fn is_supported "pus" {} -> bool
        },
        item! {
            /// Returns the metadata of the platform.
            ///
            /// This typically contains the version and side (A or B) of the running platform.
            fn metadata "pum" {
                /// Where to write the allocated metadata.
                ptr: *mut *mut u8,

                /// Where to write the metadata length.
                len: *mut usize,
            } -> ()
        },
        item! {
            /// Starts a platform update process.
            fn initialize "pui" {
                /// Zero for normal operation. One for dry-run.
                ///
                /// During a dry-run, any mutable operation is skipped and only checks are
                /// performed.
                dry_run: usize,
            } -> ()
        },
        item! {
            /// Processes the next chunk of a platform update.
            fn process "pup" {
                /// Address of the chunk.
                ptr: *const u8,

                /// Length of the chunk in bytes.
                len: usize,
            } -> ()
        },
        item! {
            /// Finalizes a platform update process.
            ///
            /// This function will reboot when the update is successful and thus only returns in
            /// case of errors or in dry-run mode.
            fn finalize "puf" {} -> ()
        },
    ];
    Item::Mod(Mod { docs, name, items })
}
