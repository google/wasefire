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

//! Fingerprint interface.

use crate::Error;

#[cfg(feature = "api-fingerprint-matcher")]
pub mod matcher;
#[cfg(feature = "api-fingerprint-sensor")]
pub mod sensor;

/// Fingerprint event.
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, PartialEq, Eq)]
pub enum Event {
    /// Fingerprint matcher event.
    #[cfg(feature = "api-fingerprint-matcher")]
    Matcher(matcher::Event),

    /// Fingerprint sensor event.
    #[cfg(feature = "api-fingerprint-sensor")]
    Sensor(sensor::Event),

    /// A finger was detected.
    FingerDetected,
}

impl<B: crate::Api> From<Event> for crate::Event<B> {
    fn from(event: Event) -> Self {
        crate::Event::Fingerprint(event)
    }
}

/// Fingerprint interface.
pub trait Api: Send {
    /// Fingerprint matcher interface.
    #[cfg(feature = "api-fingerprint-matcher")]
    type Matcher: matcher::Api;

    /// Fingerprint sensor interface.
    #[cfg(feature = "api-fingerprint-sensor")]
    type Sensor: sensor::Api;

    /// Enables finger detection.
    fn enable() -> Result<(), Error>;

    /// Disables finger detection.
    fn disable() -> Result<(), Error>;
}

/// Fingerprint matcher interface.
#[cfg(feature = "api-fingerprint-matcher")]
pub type Matcher<B> = <super::Fingerprint<B> as Api>::Matcher;

/// Fingerprint sensor interface.
#[cfg(feature = "api-fingerprint-sensor")]
pub type Sensor<B> = <super::Fingerprint<B> as Api>::Sensor;
