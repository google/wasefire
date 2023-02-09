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

use api::Api;
use board::Api as Board;

use crate::DispatchSchedulerCall;

mod button;
mod clock;
mod crypto;
mod debug;
mod led;
mod rng;
mod scheduling;
mod store;
mod usb;

pub fn process<B: Board>(call: Api<DispatchSchedulerCall<B>>) {
    match call {
        Api::Button(call) => button::process(call),
        Api::Clock(call) => clock::process(call),
        Api::Crypto(call) => crypto::process(call),
        Api::Debug(call) => debug::process(call),
        Api::Led(call) => led::process(call),
        Api::Rng(call) => rng::process(call),
        Api::Scheduling(call) => scheduling::process(call),
        Api::Store(call) => store::process(call),
        Api::Syscall(_) => todo!(),
        Api::Usb(call) => usb::process(call),
    }
}
