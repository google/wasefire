// Copyright 2024 Google LLC
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

/// Wasefire version of `std::process::Termination`.
pub trait Termination {
    /// Reports the termination at the applet level.
    ///
    /// There is nothing to report to the platform, which is why the function doesn't return
    /// anything, compared to the `std` version.
    fn report(self);
}

impl Termination for () {
    fn report(self) {}
}

impl Termination for ! {
    fn report(self) {}
}

impl Termination for Result<(), crate::Error> {
    fn report(self) {
        self.unwrap();
    }
}
