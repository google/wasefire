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

//! Timer interface.
//!
//! A timer triggers an event after a given amount of time (possibly periodically).

use crate::Error;

/// Timer event.
#[derive(Debug, PartialEq, Eq)]
pub struct Event {
    /// The timer that triggered the event.
    pub timer: usize,
}

impl From<Event> for crate::Event {
    fn from(event: Event) -> Self {
        crate::Event::Timer(event)
    }
}

/// Timer interface.
pub trait Api {
    /// Returns how many timers are available.
    ///
    /// Timers are identified by an integer smaller than this value.
    fn count(&mut self) -> usize;

    /// Arms a timer to trigger according to a command.
    fn arm(&mut self, timer: usize, command: &Command) -> Result<(), Error>;

    /// Disarms a timer regardless of whether it already triggered.
    ///
    /// The timer won't trigger further events.
    fn disarm(&mut self, timer: usize) -> Result<(), Error>;
}

impl Api for ! {
    fn count(&mut self) -> usize {
        unreachable!()
    }

    fn arm(&mut self, _: usize, _: &Command) -> Result<(), Error> {
        unreachable!()
    }

    fn disarm(&mut self, _: usize) -> Result<(), Error> {
        unreachable!()
    }
}

impl Api for () {
    fn count(&mut self) -> usize {
        0
    }

    fn arm(&mut self, _: usize, _: &Command) -> Result<(), Error> {
        Err(Error::User)
    }

    fn disarm(&mut self, _: usize) -> Result<(), Error> {
        Err(Error::User)
    }
}

#[derive(Debug, Clone)]
pub struct Command {
    /// Whether the timer should periodically trigger.
    pub periodic: bool,

    /// Duration in milliseconds after which the timer should trigger.
    pub duration_ms: usize,
}
