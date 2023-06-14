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

//! Button interface.
//!
//! A button is an input interface with 2 states: pressed and released. Buttons must support
//! triggering events when changing state. Events may be enabled or disabled per button.

use derivative::Derivative;

use crate::{Error, Id, Support, Unsupported};

/// Button event.
#[derive(Derivative)]
#[derivative(Debug(bound = ""), PartialEq(bound = ""), Eq(bound = ""))]
pub struct Event<B: crate::Api + ?Sized> {
    /// The button that triggered the event.
    pub button: Id<crate::Button<B>>,

    /// Whether the event was a button press or release.
    pub pressed: bool,
}

impl<B: crate::Api> From<Event<B>> for crate::Event<B> {
    fn from(event: Event<B>) -> Self {
        crate::Event::Button(event)
    }
}

/// Button interface.
pub trait Api: Support<usize> {
    /// Enables events for a given button.
    fn enable(button: Id<Self>) -> Result<(), Error>;

    /// Disables events for a given button.
    fn disable(button: Id<Self>) -> Result<(), Error>;
}

impl Api for Unsupported {
    fn enable(_: Id<Self>) -> Result<(), Error> {
        unreachable!()
    }

    fn disable(_: Id<Self>) -> Result<(), Error> {
        unreachable!()
    }
}
