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
#![feature(doc_auto_cfg)]
#![feature(never_type)]

extern crate alloc;

use core::marker::PhantomData;
use core::ops::Deref;

use derivative::Derivative;
pub use wasefire_error::Error;

#[cfg(feature = "api-button")]
pub mod button;
#[cfg(feature = "internal-api-crypto")]
pub mod crypto;
pub mod debug;
#[cfg(feature = "api-gpio")]
pub mod gpio;
#[cfg(feature = "api-led")]
pub mod led;
#[cfg(feature = "internal-api-platform")]
pub mod platform;
#[cfg(feature = "internal-api-radio")]
pub mod radio;
#[cfg(feature = "api-rng")]
pub mod rng;
#[cfg(feature = "api-storage")]
mod storage;
#[cfg(feature = "api-timer")]
pub mod timer;
#[cfg(feature = "api-uart")]
pub mod uart;
#[cfg(feature = "internal-api-usb")]
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
    fn syscall(_x1: u32, _x2: u32, _x3: u32, _x4: u32) -> Option<Result<u32, Error>> {
        None
    }

    #[cfg(feature = "api-button")]
    type Button: button::Api;
    #[cfg(feature = "internal-api-crypto")]
    type Crypto: crypto::Api;
    type Debug: debug::Api;
    #[cfg(feature = "api-gpio")]
    type Gpio: gpio::Api;
    #[cfg(feature = "api-led")]
    type Led: led::Api;
    #[cfg(feature = "internal-api-platform")]
    type Platform: platform::Api;
    #[cfg(feature = "internal-api-radio")]
    type Radio: radio::Api;
    #[cfg(feature = "api-rng")]
    type Rng: rng::Api;
    #[cfg(feature = "api-storage")]
    type Storage: Singleton + wasefire_store::Storage + Send;
    #[cfg(feature = "api-timer")]
    type Timer: timer::Api;
    #[cfg(feature = "api-uart")]
    type Uart: uart::Api;
    #[cfg(feature = "internal-api-usb")]
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
    #[cfg(feature = "api-button")]
    Button(button::Event<B>),

    /// Radio event.
    #[cfg(feature = "internal-api-radio")]
    Radio(radio::Event),

    /// Timer event.
    #[cfg(feature = "api-timer")]
    Timer(timer::Event<B>),

    /// UART event.
    #[cfg(feature = "api-uart")]
    Uart(uart::Event<B>),

    /// USB event.
    #[cfg(feature = "internal-api-usb")]
    Usb(usb::Event),

    /// Dummy event for typing purposes.
    Impossible(Impossible<B>),
}

/// Impossible type with board parameter.
///
/// This type is useful when the type parameter `B` needs to be mentioned in an enum. This type can
/// be destructed by calling its unreachable method.
#[derive(Derivative)]
#[derivative(Debug(bound = ""), Copy(bound = ""), Hash(bound = ""))]
#[derivative(PartialEq(bound = ""), Eq(bound = ""), Ord(bound = ""))]
#[derivative(Ord = "feature_allow_slow_enum")]
pub struct Impossible<B: Api + ?Sized>(Void, PhantomData<B>);

// TODO(https://github.com/mcarton/rust-derivative/issues/112): Use Clone(bound = "") instead.
impl<B: Api + ?Sized> Clone for Impossible<B> {
    fn clone(&self) -> Self {
        *self
    }
}

// TODO(https://github.com/mcarton/rust-derivative/issues/112): Use PartialOrd(bound = "") instead.
impl<B: Api + ?Sized> PartialOrd for Impossible<B> {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<B: Api + ?Sized> Impossible<B> {
    pub fn unreachable(&self) -> ! {
        match self.0 {}
    }
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
enum Void {}

#[cfg(feature = "api-button")]
pub type Button<B> = <B as Api>::Button;
#[cfg(feature = "internal-api-crypto")]
pub type Crypto<B> = <B as Api>::Crypto;
pub type Debug<B> = <B as Api>::Debug;
#[cfg(feature = "api-gpio")]
pub type Gpio<B> = <B as Api>::Gpio;
#[cfg(feature = "api-led")]
pub type Led<B> = <B as Api>::Led;
#[cfg(feature = "internal-api-platform")]
pub type Platform<B> = <B as Api>::Platform;
#[cfg(feature = "internal-api-radio")]
pub type Radio<B> = <B as Api>::Radio;
#[cfg(feature = "api-rng")]
pub type Rng<B> = <B as Api>::Rng;
#[cfg(feature = "api-storage")]
pub type Storage<B> = <B as Api>::Storage;
#[cfg(feature = "api-timer")]
pub type Timer<B> = <B as Api>::Timer;
#[cfg(feature = "api-uart")]
pub type Uart<B> = <B as Api>::Uart;
#[cfg(feature = "internal-api-usb")]
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

            #[cfg(feature = "api-button")]
            type Button = Unsupported;
            #[cfg(feature = "internal-api-crypto")]
            type Crypto = Unsupported;
            type Debug = Unsupported;
            #[cfg(feature = "api-gpio")]
            type Gpio = Unsupported;
            #[cfg(feature = "api-led")]
            type Led = Unsupported;
            #[cfg(feature = "internal-api-platform")]
            type Platform = Unsupported;
            #[cfg(feature = "internal-api-radio")]
            type Radio = Unsupported;
            #[cfg(feature = "api-rng")]
            type Rng = Unsupported;
            #[cfg(feature = "api-storage")]
            type Storage = Unsupported;
            #[cfg(feature = "api-timer")]
            type Timer = Unsupported;
            #[cfg(feature = "api-uart")]
            type Uart = Unsupported;
            #[cfg(feature = "internal-api-usb")]
            type Usb = Unsupported;
        }
    }
}
