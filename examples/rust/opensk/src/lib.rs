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

//! Example applet running OpenSK.

#![no_std]
wasefire::applet!();

use opensk_lib::Ctap;

mod env;

fn main() -> ! {
    let mut _ctap = Ctap::new(env::WasefireEnv);
    debug!("OpenSK initialized");
    // usb::ctap::Listener::new();
    todo!()
}
