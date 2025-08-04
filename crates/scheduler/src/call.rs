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

use wasefire_applet_api::{self as api, Api};
use wasefire_board_api::Api as Board;

use crate::{DispatchSchedulerCall, SchedulerCall};

#[cfg_attr(not(feature = "applet-api-store"), allow(unused_macros))]
macro_rules! or_fail {
    ($feature:literal, $name:ident($call:ident)) => {{
        #[cfg(feature = $feature)]
        $name($call);
        #[cfg(not(feature = $feature))]
        $call
            .reply_(Err(wasefire_error::Error::world(wasefire_error::Code::NotImplemented).into()));
    }};
}

#[cfg_attr(not(feature = "applet-api-crypto-ec"), allow(unused_macros))]
macro_rules! or_false {
    ($feature:literal, $support:expr) => {{
        #[cfg(feature = $feature)]
        let support = $support;
        #[cfg(not(feature = $feature))]
        let support = false;
        support
    }};
}

#[cfg_attr(not(feature = "applet-api-crypto-hash"), allow(unused_macros))]
macro_rules! trap_use {
    ($($x:ident),* $(,)?) => {{
        $( let _ = $x; )*
        Err(crate::Trap)?
    }};
}

#[cfg(feature = "applet-api-button")]
mod button;
#[cfg(feature = "applet-api-clock")]
mod clock;
#[cfg(feature = "internal-applet-api-crypto")]
mod crypto;
mod debug;
#[cfg(feature = "internal-applet-api-fingerprint")]
mod fingerprint;
#[cfg(feature = "applet-api-gpio")]
mod gpio;
#[cfg(feature = "applet-api-led")]
mod led;
#[cfg(feature = "internal-applet-api-platform")]
mod platform;
#[cfg(feature = "internal-applet-api-radio")]
mod radio;
#[cfg(feature = "applet-api-rng")]
mod rng;
mod scheduling;
#[cfg(feature = "internal-applet-api-store")]
mod store;
#[cfg(feature = "applet-api-timer")]
mod timer;
#[cfg(feature = "applet-api-uart")]
mod uart;
#[cfg(feature = "internal-applet-api-usb")]
mod usb;

pub fn process<B: Board>(call: Api<DispatchSchedulerCall<B>>) {
    match call {
        #[cfg(feature = "applet-api-button")]
        Api::Button(call) => button::process(call),
        #[cfg(feature = "applet-api-clock")]
        Api::Clock(call) => clock::process(call),
        #[cfg(feature = "internal-applet-api-crypto")]
        Api::Crypto(call) => crypto::process(call),
        Api::Debug(call) => debug::process(call),
        #[cfg(feature = "internal-applet-api-fingerprint")]
        Api::Fingerprint(call) => fingerprint::process(call),
        #[cfg(feature = "applet-api-gpio")]
        Api::Gpio(call) => gpio::process(call),
        #[cfg(feature = "applet-api-led")]
        Api::Led(call) => led::process(call),
        #[cfg(feature = "internal-applet-api-platform")]
        Api::Platform(call) => platform::process(call),
        #[cfg(feature = "internal-applet-api-radio")]
        Api::Radio(call) => radio::process(call),
        #[cfg(feature = "applet-api-rng")]
        Api::Rng(call) => rng::process(call),
        Api::Scheduling(call) => scheduling::process(call),
        #[cfg(feature = "internal-applet-api-store")]
        Api::Store(call) => store::process(call),
        Api::Syscall(call) => syscall(call),
        #[cfg(feature = "applet-api-timer")]
        Api::Timer(call) => timer::process(call),
        #[cfg(feature = "applet-api-uart")]
        Api::Uart(call) => uart::process(call),
        #[cfg(feature = "internal-applet-api-usb")]
        Api::Usb(call) => usb::process(call),
    }
}

fn syscall<B: Board>(mut call: SchedulerCall<B, api::syscall::Sig>) {
    let api::syscall::Params { x1, x2, x3, x4 } = call.read();
    let result = B::vendor(call.memory(), *x1, *x2, *x3, *x4);
    call.reply(result);
}
