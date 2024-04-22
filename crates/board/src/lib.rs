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
use wasefire_error::Code;
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
    /// implementation traps by returning `None`. The platform will panic if `Some(Ok(x))` is
    /// returned when `x as i32` would be negative.
    fn syscall(_x1: u32, _x2: u32, _x3: u32, _x4: u32) -> Option<Result<u32, Error>> {
        None
    }

    /// Button interface.
    #[cfg(feature = "api-button")]
    type Button: button::Api;

    /// Cryptography interface.
    #[cfg(feature = "internal-api-crypto")]
    type Crypto: crypto::Api;

    /// Debugging and testing interface.
    type Debug: debug::Api;

    /// Low-level GPIO interface.
    #[cfg(feature = "api-gpio")]
    type Gpio: gpio::Api;

    /// LED interface.
    #[cfg(feature = "api-led")]
    type Led: led::Api;

    /// Platform interface.
    #[cfg(feature = "internal-api-platform")]
    type Platform: platform::Api;

    /// Radio interface.
    #[cfg(feature = "internal-api-radio")]
    type Radio: radio::Api;

    /// Random number generator interface.
    #[cfg(feature = "api-rng")]
    type Rng: rng::Api;

    /// Persistent storage interface.
    #[cfg(feature = "api-storage")]
    type Storage: Singleton + wasefire_store::Storage + Send;

    /// Timer interface.
    #[cfg(feature = "api-timer")]
    type Timer: timer::Api;

    /// UART interface.
    #[cfg(feature = "api-uart")]
    type Uart: uart::Api;

    /// USB interface.
    #[cfg(feature = "internal-api-usb")]
    type Usb: usb::Api;
}

/// Describes how an API is supported.
///
/// The `Value` type parameter is usually `bool`. It may also be `usize` for APIs that have multiple
/// similar things like buttons, leds, and timers.
pub trait Support<Value> {
    /// Whether and how the API is supported.
    const SUPPORT: Value;
}

/// Marker trait for supported API.
pub trait Supported {}

/// Provides access to a singleton API.
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
    /// Provides a static proof of dead code.
    pub fn unreachable(&self) -> ! {
        match self.0 {}
    }
}

#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
enum Void {}

/// Button interface.
#[cfg(feature = "api-button")]
pub type Button<B> = <B as Api>::Button;

/// Cryptography interface.
#[cfg(feature = "internal-api-crypto")]
pub type Crypto<B> = <B as Api>::Crypto;

/// Debugging and testing interface.
pub type Debug<B> = <B as Api>::Debug;

/// Low-level GPIO interface.
#[cfg(feature = "api-gpio")]
pub type Gpio<B> = <B as Api>::Gpio;

/// LED interface.
#[cfg(feature = "api-led")]
pub type Led<B> = <B as Api>::Led;

/// Platform interface.
#[cfg(feature = "internal-api-platform")]
pub type Platform<B> = <B as Api>::Platform;

/// Radio interface.
#[cfg(feature = "internal-api-radio")]
pub type Radio<B> = <B as Api>::Radio;

/// Random number generator interface.
#[cfg(feature = "api-rng")]
pub type Rng<B> = <B as Api>::Rng;

/// Persistent storage interface.
#[cfg(feature = "api-storage")]
pub type Storage<B> = <B as Api>::Storage;

/// Timer interface.
#[cfg(feature = "api-timer")]
pub type Timer<B> = <B as Api>::Timer;

/// UART interface.
#[cfg(feature = "api-uart")]
pub type Uart<B> = <B as Api>::Uart;

/// USB interface.
#[cfg(feature = "internal-api-usb")]
pub type Usb<B> = <B as Api>::Usb;

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
    /// Creates a safe identifier for an API with countable support.
    pub fn new(value: usize) -> Result<Self, Error> {
        match value < T::SUPPORT {
            true => Ok(Self { value, count: PhantomData }),
            false => Err(Error::user(Code::InvalidArgument)),
        }
    }
}

impl<T: Support<usize>> Deref for Id<T> {
    type Target = usize;

    fn deref(&self) -> &Self::Target {
        &self.value
    }
}

impl<T: Supported> Support<bool> for T {
    const SUPPORT: bool = true;
}
