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

use core::fmt::Debug;

pub mod button;
pub mod crypto;
pub mod led;
pub mod rng;
pub mod timer;
pub mod usb;

/// Board interface.
pub trait Api: button::Api + crypto::Api + led::Api + rng::Api + timer::Api + usb::Api {
    type Storage: store::Storage;

    /// Returns the oldest triggered event, if any.
    ///
    /// This function is non-blocking. See [`Self::wait_event()`] for a blocking version.
    fn try_event(&mut self) -> Option<Event>;

    /// Returns the oldest triggered event, possibly waiting until one triggers.
    ///
    /// This function is non-blocking if an event already triggered. However, if there are no event
    /// available, this function blocks and enters a power-saving state until an event triggers.
    fn wait_event(&mut self) -> Event;

    /// Takes the storage from the board.
    ///
    /// This function returns `Some` at most once and if it does, it does so on the first call.
    fn take_storage(&mut self) -> Option<Self::Storage>;
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
