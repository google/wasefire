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

//! Board interface.
//!
//! A board provides multiple interfaces under a single [`Api`] trait. Some of these interfaces
//! support triggering [events][Event].

#![no_std]
#![feature(never_type)]

use core::fmt::Debug;

use wasefire_store as store;

pub mod button;
pub mod crypto;
pub mod debug;
pub mod led;
pub mod rng;
pub mod timer;
pub mod usb;

/// Board interface.
///
/// Associated types have predefined implementations:
/// - `!` (the never type) implements an API by panicking (the accessor function cannot be
///   implemented and must itself panic)
/// - `()` (the unit type) implements an API for something countable by using zero.
pub trait Api {
    /// Returns the oldest triggered event, if any.
    ///
    /// This function is non-blocking. See [`Self::wait_event()`] for a blocking version.
    fn try_event(&mut self) -> Option<Event>;

    /// Returns the oldest triggered event, possibly waiting until one triggers.
    ///
    /// This function is non-blocking if an event already triggered. However, if there are no event
    /// available, this function blocks and enters a power-saving state until an event triggers.
    fn wait_event(&mut self) -> Event;

    /// Storage type.
    type Storage: store::Storage;

    /// Takes the storage from the board.
    ///
    /// This function returns `Some` at most once and if it does, it does so on the first call.
    fn take_storage(&mut self) -> Option<Self::Storage>;

    type Button<'a>: button::Api
    where Self: 'a;
    fn button(&mut self) -> Self::Button<'_>;

    type Crypto<'a>: crypto::Api
    where Self: 'a;
    fn crypto(&mut self) -> Self::Crypto<'_>;

    type Debug<'a>: debug::Api
    where Self: 'a;
    fn debug(&mut self) -> Self::Debug<'_>;

    type Led<'a>: led::Api
    where Self: 'a;
    fn led(&mut self) -> Self::Led<'_>;

    type Rng<'a>: rng::Api
    where Self: 'a;
    fn rng(&mut self) -> Self::Rng<'_>;

    type Timer<'a>: timer::Api
    where Self: 'a;
    fn timer(&mut self) -> Self::Timer<'_>;

    type Usb<'a>: usb::Api
    where Self: 'a;
    fn usb(&mut self) -> Self::Usb<'_>;
}

/// Events that interfaces may trigger.
///
/// Events are de-duplicated if the previous one was not processed yet, because some events may
/// trigger repeatedly.
#[derive(Debug, PartialEq, Eq)]
pub enum Event {
    /// Button event.
    Button(button::Event),

    /// Timer event.
    Timer(timer::Event),

    /// USB event.
    Usb(usb::Event),
}

/// Errors that interfaces may return.
///
/// Because a board interfaces between the user and the world, there's 2 types of errors: those due
/// to the user and those due to the world. If the board itself errors, the error should be handled
/// internally: either by an automatic reset (in production) or by halting execution until a manual
/// reset (during testing to permit debugging).
#[derive(Debug, Copy, Clone)]
pub enum Error {
    /// The user made an error.
    User,

    /// The world made an error.
    World,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_impl() {
        struct Empty;
        impl Api for Empty {
            fn try_event(&mut self) -> Option<Event> {
                todo!()
            }

            fn wait_event(&mut self) -> Event {
                todo!()
            }

            type Storage = !;
            fn take_storage(&mut self) -> Option<Self::Storage> {
                todo!()
            }

            type Button<'a> = ();
            fn button(&mut self) -> Self::Button<'_> {
                ()
            }

            type Crypto<'a> = !;
            fn crypto(&mut self) -> Self::Crypto<'_> {
                todo!()
            }

            type Debug<'a> = !;
            fn debug(&mut self) -> Self::Debug<'_> {
                todo!()
            }

            type Led<'a> = ();
            fn led(&mut self) -> Self::Led<'_> {
                ()
            }

            type Rng<'a> = !;
            fn rng(&mut self) -> Self::Rng<'_> {
                todo!()
            }

            type Timer<'a> = ();
            fn timer(&mut self) -> Self::Timer<'_> {
                ()
            }

            type Usb<'a> = !;
            fn usb(&mut self) -> Self::Usb<'_> {
                todo!()
            }
        }
    }
}
