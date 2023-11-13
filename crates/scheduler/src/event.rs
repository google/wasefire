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

use derivative::Derivative;
use wasefire_board_api::{Api as Board, Event};
#[cfg(feature = "wasm")]
pub use wasefire_interpreter::InstId;
use wasefire_logger as log;

use crate::Scheduler;

pub mod button;
pub mod timer;
pub mod uart;
pub mod usb;

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[cfg(feature = "native")]
pub struct InstId;

// TODO: This could be encoded into a u32 for performance/footprint.
#[derive(Derivative)]
#[derivative(Debug(bound = ""), Copy(bound = ""), Hash(bound = ""))]
#[derivative(PartialEq(bound = ""), Eq(bound = ""), Ord(bound = ""))]
#[derivative(Ord = "feature_allow_slow_enum")]
pub enum Key<B: Board> {
    Button(button::Key<B>),
    Timer(timer::Key<B>),
    Uart(uart::Key<B>),
    Usb(usb::Key),
}

// TODO(https://github.com/mcarton/rust-derivative/issues/112): Use Clone(bound = "") instead.
impl<B: Board> Clone for Key<B> {
    fn clone(&self) -> Self {
        *self
    }
}

// TODO(https://github.com/mcarton/rust-derivative/issues/112): Use PartialOrd(bound = "") instead.
impl<B: Board> PartialOrd for Key<B> {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<'a, B: Board> From<&'a Event<B>> for Key<B> {
    fn from(event: &'a Event<B>) -> Self {
        match event {
            Event::Button(event) => Key::Button(event.into()),
            Event::Timer(event) => Key::Timer(event.into()),
            Event::Uart(event) => Key::Uart(event.into()),
            Event::Usb(event) => Key::Usb(event.into()),
        }
    }
}

#[derive(Derivative)]
#[derivative(Debug(bound = ""), Clone(bound = ""))]
#[derivative(PartialEq(bound = ""), Eq(bound = ""), Ord(bound = ""))]
#[derivative(Ord = "feature_allow_slow_enum")]
pub struct Handler<B: Board> {
    pub key: Key<B>,
    pub inst: InstId,
    pub func: u32,
    pub data: u32,
}

// TODO(https://github.com/mcarton/rust-derivative/issues/112): Use PartialOrd(bound = "") instead.
impl<B: Board> PartialOrd for Handler<B> {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<B: Board> Borrow<Key<B>> for Handler<B> {
    fn borrow(&self) -> &Key<B> {
        &self.key
    }
}

#[cfg_attr(feature = "native", allow(clippy::needless_pass_by_ref_mut))]
pub fn process<B: Board>(scheduler: &mut Scheduler<B>, event: Event<B>) {
    let Handler { inst, func, data, .. } = match scheduler.applet.get(Key::from(&event)) {
        Some(x) => x,
        None => {
            // This should not happen because we remove pending events when disabling an event.
            log::error!("Missing handler for event.");
            return;
        }
    };
    let mut params = vec![*func, *data];
    match event {
        Event::Button(event) => button::process(event, &mut params),
        Event::Timer(_) => timer::process(),
        Event::Uart(_) => uart::process(),
        Event::Usb(event) => usb::process(event),
    }
    #[cfg(feature = "wasm")]
    {
        use alloc::format;
        let name = format!("cb{}", params.len() - 2);
        scheduler.call(*inst, &name, &params);
    }
    #[cfg(feature = "native")]
    {
        use alloc::boxed::Box;

        use crate::native;

        #[derive(Copy, Clone)]
        #[repr(transparent)]
        struct U8(*const u8);
        unsafe impl Send for U8 {}

        let InstId = inst;
        macro_rules! schedule {
            ($cb:ident($($x:ident),*)) => { schedule!(($($x)*) (2) [] $cb) };

            (($x:ident $($ys:ident)*) ($i:expr) [$($xs:ident: $ts:ty = $is:expr,)*] $cb:ident) => {
                schedule!(($($ys)*) ($i + 1) [$($xs: $ts = $is,)* $x: usize = $i,] $cb)
            };

            (() $count:tt $args:tt $cb:ident) => { schedule!($args $cb) };

            ([$($x:ident: $t:ty = $i:expr,)*] $cb:ident) => {{
                extern "C" {
                    fn $cb(ptr: extern "C" fn(U8 $(, $t)*), this: U8 $(, $x: $t)*);
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
