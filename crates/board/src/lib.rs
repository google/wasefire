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
//! A board provides hierarchy of interfaces under the root [`Api`] trait. Some of these interfaces
//! support triggering [events][Event].

#![no_std]
#![feature(never_type)]

extern crate alloc;

use core::marker::PhantomData;
use core::ops::Deref;

use derivative::Derivative;

pub mod button;
pub mod crypto;
pub mod debug;
pub mod gpio;
pub mod led;
pub mod platform;
pub mod radio;
pub mod rng;
mod storage;
pub mod timer;
pub mod uart;
pub mod usb;

/// Board interface.
///
/// This is essentially a type hierarchy. The implementation is responsible for handling a possible
/// explicit global state. The type implementing this API may be equivalent to the never type (e.g.
/// an empty enum) because it is never used, i.e. there are no functions which take `self`.
///
/// All interfaces are implemented by [`Unsupported`]. This can be used for interfaces that don't
/// have hardware support, are not yet implemented, or should use a software implementation.
pub trait Api: Send + 'static {
    /// Returns the oldest triggered event, if any.
    ///
    /// This function is non-blocking. See [`Self::wait_event()`] for a blocking version.
    fn try_event() -> Option<Event<Self>>;

    /// Returns the oldest triggered event, possibly waiting until one triggers.
    ///
    /// This function is non-blocking if an event already triggered. However, if there are no event
    /// available, this function blocks and enters a power-saving state until an event triggers.
    fn wait_event() -> Event<Self>;

    /// Board-specific syscalls.
    ///
    /// Those calls are directly forwarded from the applet by the scheduler. The default
    /// implementation traps.
    fn syscall(_x1: u32, _x2: u32, _x3: u32, _x4: u32) -> Option<u32> {
        None
    }

    type Button: button::Api;
    type Crypto: crypto::Api;
    type Debug: debug::Api;
    type Gpio: gpio::Api;
    type Led: led::Api;
    type Platform: platform::Api;
    type Radio: radio::Api;
    type Rng: rng::Api;
    type Storage: Singleton + wasefire_store::Storage + Send;
    type Timer: timer::Api;
    type Uart: uart::Api;
    type Usb: usb::Api;
}

/// Describes how an API is supported.
///
/// The `Value` type parameter is usually `bool`. It may also be `usize` for APIs that have multiple
/// similar things like buttons, leds, and timers.
pub trait Support<Value> {
    const SUPPORT: Value;
}

/// Marker trait for supported API.
pub trait Supported {}

/// Provides access to a (possibly unsupported) singleton API.
pub trait Singleton: Sized {
    /// Returns the singleton.
    ///
    /// Returns `None` if the API is not supported. Returns `None` if called more than once.
    fn take() -> Option<Self>;
}

/// Events that interfaces may trigger.
///
/// Events are de-duplicated if the previous one was not processed yet, because some events may
/// trigger repeatedly.
#[derive(Derivative)]
#[derivative(Debug(bound = ""), PartialEq(bound = ""), Eq(bound = ""))]
pub enum Event<B: Api + ?Sized> {
    /// Button event.
    Button(button::Event<B>),

    /// Radio event.
    Radio(radio::Event),

    /// Timer event.
    Timer(timer::Event<B>),

    /// UART event.
    Uart(uart::Event<B>),

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

pub type Button<B> = <B as Api>::Button;
pub type Crypto<B> = <B as Api>::Crypto;
pub type Debug<B> = <B as Api>::Debug;
pub type Gpio<B> = <B as Api>::Gpio;
pub type Led<B> = <B as Api>::Led;
pub type Platform<B> = <B as Api>::Platform;
pub type Radio<B> = <B as Api>::Radio;
pub type Rng<B> = <B as Api>::Rng;
pub type Storage<B> = <B as Api>::Storage;
pub type Timer<B> = <B as Api>::Timer;
pub type Uart<B> = <B as Api>::Uart;
pub type Usb<B> = <B as Api>::Usb;

/// Unsupported interface.
#[derive(Debug)]
pub enum Unsupported {}

/// Valid identifier for a countable API.
#[derive(Derivative)]
#[derivative(Debug(bound = ""), Copy(bound = ""), Hash(bound = ""))]
#[derivative(PartialEq(bound = ""), Eq(bound = ""), Ord(bound = ""))]
pub struct Id<T: Support<usize> + ?Sized> {
    // Invariant: value < T::SUPPORT
    value: usize,
    count: PhantomData<T>,
}

// TODO(https://github.com/mcarton/rust-derivative/issues/112): Use Clone(bound = "") instead.
impl<T: Support<usize> + ?Sized> Clone for Id<T> {
    fn clone(&self) -> Self {
        *self
    }
}

// TODO(https://github.com/mcarton/rust-derivative/issues/112): Use PartialOrd(bound = "") instead.
impl<T: Support<usize> + ?Sized> PartialOrd for Id<T> {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: Support<usize>> Id<T> {
    pub fn new(value: usize) -> Option<Self> {
        (value < T::SUPPORT).then_some(Self { value, count: PhantomData })
    }
}

impl<T: Support<usize>> Deref for Id<T> {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl Support<bool> for Unsupported {
    const SUPPORT: bool = false;
}

impl Support<usize> for Unsupported {
    const SUPPORT: usize = 0;
}

impl<T: Supported> Support<bool> for T {
    const SUPPORT: bool = true;
}

impl Singleton for Unsupported {
    fn take() -> Option<Self> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unsupported() {
        enum Test {}
        impl Api for Test {
            fn try_event() -> Option<Event<Self>> {
                todo!()
            }

            fn wait_event() -> Event<Self> {
                todo!()
            }

            type Button = Unsupported;
            type Crypto = Unsupported;
            type Debug = Unsupported;
            type Gpio = Unsupported;
            type Led = Unsupported;
            type Platform = Unsupported;
            type Radio = Unsupported;
            type Rng = Unsupported;
            type Storage = Unsupported;
            type Timer = Unsupported;
            type Uart = Unsupported;
            type Usb = Unsupported;
        }
    }
}
