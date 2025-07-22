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
    let name = "matcher".into();
    let items = vec![
        item! {
            /// Returns whether fingerprint matcher is supported.
            fn is_supported "fms" {} -> bool
        },
        item! {
            /// Returns the length of a template ID.
            ///
            /// This is a constant and all template IDs have this length.
            fn template_length "fmt" {} -> usize
        },
        item! {
            /// Starts an enrollment process.
            ///
            /// An enrollment process is over when it succeeds, fails, or is aborted.
            fn enroll "fme" {
                /// Callback for each enrollment step.
                ///
                /// This handler may be called multiple times during the enrollment process. The
                /// argument is the estimated number of remaining steps.
                ///
                /// The handler is unregistered when the enrollment process is over.
                handler_step_func: fn { data: *const void, remaining: usize },
                handler_step_data: *const void,

                /// Callback when the enrollment process finishes.
                ///
                /// This handler is called at most once. The `template` pointer is allocated by the
                /// scheduler and contains the template ID. Ownership is transferred to the handler.
                /// In case of error (`result` is negative), the pointer is null.
                ///
                /// The handler is unregistered when the enrollment process is over. When called, it
                /// is unregistered before being called.
                handler_func: fn { data: *const void, result: isize, template: *mut u8 },
                handler_data: *const void,
            } -> ()
        },
        item! {
            /// Aborts an enrollment process.
            fn abort_enroll "fmE" {} -> ()
        },
        item! {
            /// Starts an identification process.
            fn identify "fmi" {
                /// Specific template to identify against (may be null for all enrolled fingers).
                template: *const u8,

                /// Callback when the identification process finishes.
                ///
                /// The handler is called at most once. The result of the identification is encoded
                /// as follows:
                ///
                /// - `result` is 1 for a successful identification. The `template` pointer is
                /// allocated by the scheduler and contains the template ID.
                ///
                /// - `result` is 0 for a mismatch identification. The pointer is null.
                ///
                /// - `result` is negative for errors. The pointer is null.
                ///
                /// The handler is unregistered before being called, such that the handler may
                /// trigger a following identification.
                handler_func: fn { data: *const void, result: isize, template: *mut u8 },
                handler_data: *const void,
            } -> ()
        },
        item! {
            /// Aborts an identification process.
            fn abort_identify "fmI" {} -> ()
        },
        item! {
            /// Deletes a template (or all templates) of enrolled fingers.
            fn delete_template "fmd" {
                /// The template to delete (may be null for all enrolled fingers).
                template: *const u8,
            } -> ()
        },
        item! {
            /// List the templates of enrolled fingers.
            ///
            /// Returns the number of templates.
            fn list_templates "fml" {
                /// Pointer to the pointer to the templates.
                ///
                /// This pointer is written with a pointer allocated by the scheduler and containing
                /// the templates (concatenated one after the other).
                templates: *mut *mut u8,
            } -> usize
        },
    ];
    Item::Mod(Mod { docs, name, items })
}
