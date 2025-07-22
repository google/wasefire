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

pub(crate) fn new() -> Item {
    let docs = docs!();
    let name = "sensor".into();
    let items = vec![
        item! {
            /// Returns whether fingerprint sensor is supported.
            fn is_supported "fss" {} -> bool
        },
        item! {
            /// Starts an image capture process.
            fn capture "fsc" {
                /// Callback when the identification process succeeded.
                ///
                /// The handler is called at most once. The result of the capture is encoded
                /// as follows:
                ///
                /// - `width` is positive for a successful capture. The pointer is allocated by the
                /// scheduler and contains the image (length is the product of width and height).
                ///
                /// - `width` is negative for errors. The pointer is null and height is zero.
                ///
                /// The handler is unregistered before being called, such that the handler may
                /// trigger another capture.
                handler_func: fn { data: *const void, ptr: *mut u8, width: isize, height: usize },
                handler_data: *const void,
            } -> ()
        },
        item! {
            /// Aborts an image capture process.
            fn abort_capture "fsC" {} -> ()
        },
    ];
    Item::Mod(Mod { docs, name, items })
}
