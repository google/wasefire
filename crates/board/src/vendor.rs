// Copyright 2025 Google LLC
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

//! Vendor interface.

use alloc::vec::Vec;
use core::fmt::Debug;
use core::hash::Hash;

use derive_where::derive_where;
use wasefire_error::Error;
use wasefire_logger::MaybeFormat;

use crate::Failure;
use crate::applet::{Handlers, Memory};

/// Vendor event.
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive_where(Debug, PartialEq, Eq)]
pub struct Event<B: crate::Api + ?Sized>(pub <crate::Vendor<B> as Api>::Event);

impl<B: crate::Api> From<Event<B>> for crate::Event<B> {
    fn from(event: Event<B>) -> Self {
        crate::Event::Vendor(event)
    }
}

/// Vendor key.
pub type Key<B> = <super::Vendor<B> as Api>::Key;

/// Vendor interface.
pub trait Api: Send {
    /// Vendor event.
    ///
    /// Event equality is used by the scheduler. If an applet receives 2 equal events and did not
    /// process the first when the second arrives, then the second event is ignored.
    ///
    /// Use [`NoEvent`] if you don't need events.
    type Event: MaybeFormat + Debug + Eq + Send;

    /// Vendor key.
    ///
    /// The key of an even defines which applet event handler will process it.
    ///
    /// Use `()` if you don't need events.
    type Key: MaybeFormat + Debug + Copy + Hash + Ord + Send;

    /// Returns the key of an event.
    fn key(event: &Self::Event) -> Self::Key;

    /// Vendor syscall.
    ///
    /// The returned value in an encoded `Result<u32, Error>`. In particular, the `u32` must not
    /// exceed `i32::MAX` otherwise the platform will panic.
    fn syscall(
        memory: impl Memory, handlers: impl Handlers<Self::Key>, x1: u32, x2: u32, x3: u32, x4: u32,
    ) -> Result<u32, Failure>;

    /// Vendor callback.
    ///
    /// This function should convert the event into parameters to the applet handler. The number of
    /// parameters should be fixed for the event key. It cannot call `handlers.register()`.
    fn callback(
        memory: impl Memory, handlers: impl Handlers<Self::Key>, event: Self::Event,
        params: &mut Vec<u32>,
    );

    /// Disables an event.
    ///
    /// Events can be enabled (and disabled) with [`Self::syscall()`] by applets. This function is
    /// used by the scheduler when an applet dies with registered handlers. The event keys of those
    /// handlers are disabled.
    fn disable(key: Self::Key) -> Result<(), Error>;
}

/// Helper type to indicate the vendor interface does not have events.
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, PartialEq, Eq)]
pub enum NoEvent {}
