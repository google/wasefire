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

use alloc::vec;
use core::borrow::Borrow;

use derive_where::derive_where;
use wasefire_board_api::{Api as Board, Event, Impossible};
use wasefire_error::Error;
#[cfg(feature = "wasm")]
pub use wasefire_interpreter::InstId;
use wasefire_logger as log;

use crate::Scheduler;

#[cfg(feature = "board-api-button")]
pub mod button;
#[cfg(feature = "internal-board-api-fingerprint")]
pub mod fingerprint;
#[cfg(feature = "board-api-gpio")]
pub mod gpio;
pub mod platform;
#[cfg(feature = "board-api-timer")]
pub mod timer;
#[cfg(feature = "board-api-uart")]
pub mod uart;
#[cfg(feature = "internal-board-api-usb")]
pub mod usb;
#[cfg(feature = "board-api-vendor")]
pub mod vendor;

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[cfg(feature = "native")]
pub struct InstId;

// TODO: This could be encoded into a u32 for performance/footprint.
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive_where(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum Key<B: Board> {
    #[cfg(feature = "board-api-button")]
    Button(button::Key<B>),
    #[cfg(feature = "internal-board-api-fingerprint")]
    Fingerprint(fingerprint::Key),
    #[cfg(feature = "board-api-gpio")]
    Gpio(gpio::Key<B>),
    Platform(platform::Key),
    #[cfg(feature = "board-api-timer")]
    Timer(timer::Key<B>),
    #[cfg(feature = "board-api-uart")]
    Uart(uart::Key<B>),
    #[cfg(feature = "internal-board-api-usb")]
    Usb(usb::Key),
    #[cfg(feature = "board-api-vendor")]
    Vendor(vendor::Key<B>),
    _Impossible(Impossible<B>),
}

#[cfg_attr(not(feature = "board-api-button"), allow(unused_macros))]
macro_rules! or_unreachable {
    ($feature:literal, [$($event:ident)*], $expr:expr) => {{
        $( #[cfg(not(feature = $feature))] let _ = $event; )*
        #[cfg(not(feature = $feature))]
        unreachable!();
        #[cfg(feature = $feature)]
        $expr
    }};
}

impl<'a, B: Board> From<&'a Event<B>> for Key<B> {
    fn from(event: &'a Event<B>) -> Self {
        match event {
            #[cfg(feature = "board-api-button")]
            Event::Button(event) => {
                or_unreachable!("applet-api-button", [event], Key::Button(event.into()))
            }
            #[cfg(feature = "internal-board-api-fingerprint")]
            Event::Fingerprint(event) => {
                or_unreachable!(
                    "internal-applet-api-fingerprint",
                    [event],
                    Key::Fingerprint(event.into())
                )
            }
            #[cfg(feature = "board-api-gpio")]
            Event::Gpio(event) => {
                or_unreachable!("applet-api-gpio", [event], Key::Gpio(event.into()))
            }
            Event::Platform(event) => {
                or_unreachable!(
                    "internal-applet-api-platform",
                    [event],
                    Key::Platform(event.into())
                )
            }
            #[cfg(feature = "board-api-timer")]
            Event::Timer(event) => {
                or_unreachable!("applet-api-timer", [event], Key::Timer(event.into()))
            }
            #[cfg(feature = "board-api-uart")]
            Event::Uart(event) => {
                or_unreachable!("applet-api-uart", [event], Key::Uart(event.into()))
            }
            #[cfg(feature = "internal-board-api-usb")]
            Event::Usb(event) => {
                or_unreachable!("internal-applet-api-usb", [event], Key::Usb(event.into()))
            }
            #[cfg(feature = "board-api-vendor")]
            Event::Vendor(event) => {
                or_unreachable!("applet-api-vendor", [event], Key::Vendor(event.into()))
            }
            Event::Impossible(x) => x.unreachable(),
        }
    }
}

impl<B: Board> Key<B> {
    pub fn disable(self) -> Result<(), Error> {
        match self {
            #[cfg(feature = "board-api-button")]
            Key::Button(x) => x.disable(),
            #[cfg(feature = "internal-board-api-fingerprint")]
            Key::Fingerprint(x) => x.disable::<B>(),
            #[cfg(feature = "board-api-gpio")]
            Key::Gpio(x) => x.disable(),
            Key::Platform(x) => x.disable(),
            #[cfg(feature = "board-api-timer")]
            Key::Timer(x) => x.disable(),
            #[cfg(feature = "board-api-uart")]
            Key::Uart(x) => x.disable(),
            #[cfg(feature = "internal-board-api-usb")]
            Key::Usb(x) => x.disable::<B>(),
            #[cfg(feature = "board-api-vendor")]
            Key::Vendor(x) => x.disable(),
            Key::_Impossible(x) => x.unreachable(),
        }
    }
}

#[derive_where(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Handler<B: Board> {
    pub key: Key<B>,
    pub inst: InstId,
    pub func: u32,
    pub data: u32,
}

impl<B: Board> Borrow<Key<B>> for Handler<B> {
    fn borrow(&self) -> &Key<B> {
        &self.key
    }
}

#[cfg_attr(feature = "native", allow(clippy::needless_pass_by_ref_mut))]
pub fn process<B: Board>(scheduler: &mut Scheduler<B>, event: Event<B>) {
    let applet = scheduler.applet.get().unwrap();
    let (inst, func, data) = match applet.get(Key::from(&event)) {
        Some(x) => (x.inst, x.func, x.data),
        None => {
            // This should not happen because we remove pending events when disabling an event.
            log::error!("Missing handler for event.");
            return;
        }
    };
    let mut params = vec![func, data];
    let _ = (&inst, &mut params); // in case there are no events
    match event {
        #[cfg(feature = "board-api-button")]
        Event::Button(event) => {
            or_unreachable!("applet-api-button", [event], button::process(event, &mut params))
        }
        #[cfg(feature = "internal-board-api-fingerprint")]
        Event::Fingerprint(event) => {
            or_unreachable!(
                "internal-applet-api-fingerprint",
                [event],
                fingerprint::process(event, &mut params, applet)
            )
        }
        #[cfg(feature = "board-api-gpio")]
        Event::Gpio(_) => or_unreachable!("applet-api-gpio", [], gpio::process()),
        Event::Platform(event) => platform::process(event),
        #[cfg(feature = "board-api-timer")]
        Event::Timer(_) => or_unreachable!("applet-api-timer", [], timer::process()),
        #[cfg(feature = "board-api-uart")]
        Event::Uart(_) => or_unreachable!("applet-api-uart", [], uart::process()),
        #[cfg(feature = "internal-board-api-usb")]
        Event::Usb(event) => {
            or_unreachable!("internal-applet-api-usb", [event], usb::process(event))
        }
        #[cfg(feature = "board-api-vendor")]
        Event::Vendor(event) => {
            or_unreachable!(
                "applet-api-vendor",
                [event],
                vendor::process(event, &mut params, applet)
            )
        }
        Event::Impossible(x) => x.unreachable(),
    }
    #[allow(unreachable_code)] // when there are no events
    #[cfg(feature = "wasm")]
    {
        use alloc::format;
        let name = format!("cb{}", params.len() - 2);
        scheduler.call(inst, &name, &params);
    }
    #[allow(unreachable_code)] // when there are no events
    #[cfg(feature = "native")]
    {
        use alloc::boxed::Box;

        use crate::native;

        #[derive(Copy, Clone)]
        #[repr(transparent)]
        struct U8(#[allow(dead_code)] *const u8);
        unsafe impl Send for U8 {}

        let InstId = inst;
        macro_rules! schedule {
            ($cb:ident($($x:ident),*)) => { schedule!(($($x)*) (2) [] $cb) };

            (($x:ident $($ys:ident)*) ($i:expr) [$($xs:ident: $ts:ty = $is:expr,)*] $cb:ident) => {
                schedule!(($($ys)*) ($i + 1) [$($xs: $ts = $is,)* $x: usize = $i,] $cb)
            };

            (() $count:tt $args:tt $cb:ident) => { schedule!($args $cb) };

            ([$($x:ident: $t:ty = $i:expr,)*] $cb:ident) => {{
                unsafe extern "C" {
                    unsafe fn $cb(ptr: extern "C" fn(U8 $(, $t)*), this: U8 $(, $x: $t)*);
                }
                let ptr = unsafe {
                    core::mem::transmute::<*const u8, extern "C" fn(U8 $(, $t)*)>(params[0] as _)
                };
                let this = U8(params[1] as _);
                $( let $x = params[$i] as usize; )*
                log::debug!("Schedule callback {}{:?}.", stringify!($cb), params.as_slice());
                native::schedule_callback(Box::new(move || unsafe { $cb(ptr, this $(, $x)*) }));
            }};
        }
        match params.len() - 2 {
            0 => schedule!(applet_cb0()),
            1 => schedule!(applet_cb1(x0)),
            2 => schedule!(applet_cb2(x0, x1)),
            3 => schedule!(applet_cb3(x0, x1, x2)),
            _ => unimplemented!(),
        }
    }
}
