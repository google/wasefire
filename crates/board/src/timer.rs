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

use derivative::Derivative;

use crate::{Error, Id, Support, Unsupported};

/// Timer event.
#[derive(Derivative)]
#[derivative(Debug(bound = ""), PartialEq(bound = ""), Eq(bound = ""))]
pub struct Event<B: crate::Api + ?Sized> {
    /// The timer that triggered the event.
    pub timer: Id<crate::Timer<B>>,
}

impl<B: crate::Api> From<Event<B>> for crate::Event<B> {
    fn from(event: Event<B>) -> Self {
        crate::Event::Timer(event)
    }
}

/// Timer interface.
pub trait Api: Support<usize> {
    /// Arms a timer to trigger according to a command.
    fn arm(timer: Id<Self>, command: &Command) -> Result<(), Error>;

    /// Disarms a timer regardless of whether it already triggered.
    ///
    /// The timer won't trigger further events.
    fn disarm(timer: Id<Self>) -> Result<(), Error>;
}

impl Api for Unsupported {
    fn arm(_: Id<Self>, _: &Command) -> Result<(), Error> {
        unreachable!()
    }

    fn disarm(_: Id<Self>) -> Result<(), Error> {
        unreachable!()
    }
}

#[derive(Debug, Clone)]
pub struct Command {
    /// Whether the timer should periodically trigger.
    pub periodic: bool,

    /// Duration in milliseconds after which the timer should trigger.
    pub duration_ms: usize,
}
