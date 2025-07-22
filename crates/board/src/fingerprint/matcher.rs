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

//! Fingerprint matcher interface.

use alloc::vec::Vec;

use crate::{Error, Support};

/// Fingerprint matcher event.
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, PartialEq, Eq)]
pub enum Event {
    /// The enrollment process finished successfully.
    ///
    /// The `read_enroll()` function must be called before any other process.
    EnrollDone,

    /// The enrollment process made a step.
    EnrollStep {
        /// Estimated number of remaining steps.
        remaining: usize,
    },

    /// The enrollment process encountered an error.
    EnrollError {
        /// The device error.
        error: Error,
    },

    /// The identification process finished successfully.
    ///
    /// If the result is `true`, then the `read_identify()` function must be called before any other
    /// process to read the template ID.
    IdentifyDone {
        /// Whether the finger was identified.
        result: bool,
    },

    /// The identification process encountered an error.
    IdentifyError {
        /// The device error.
        error: Error,
    },
}

impl<B: crate::Api> From<Event> for crate::Event<B> {
    fn from(event: Event) -> Self {
        super::Event::Matcher(event).into()
    }
}

/// Fingerprint matcher interface.
pub trait Api: Support<bool> + Send {
    /// Size in bytes of a template ID.
    const TEMPLATE_ID_SIZE: usize;

    /// Starts a finger enrollment process.
    ///
    /// The process will send a (possibly empty) series of `EnrollStep` events followed by an
    /// `EnrollDone` (resp. `EnrollError`) event in case of success (resp. error) to indicate that
    /// the enrollment finished.
    fn start_enroll() -> Result<(), Error>;

    /// Reads the template ID of a successful enrollment.
    fn read_enroll(template_id: &mut [u8]) -> Result<(), Error>;

    /// Aborts a finger enrollment process.
    fn abort_enroll() -> Result<(), Error>;

    /// Starts a finger identification process.
    ///
    /// If the template ID is not provided, the identification is done against all enrolled fingers.
    ///
    /// The process will send an `IdentifyDone` (resp. `IdentifyError`) event in case of success
    /// (resp. error) to indicate that the identification finished.
    fn start_identify(template_id: Option<&[u8]>) -> Result<(), Error>;

    /// Reads the template ID of an identified finger.
    fn read_identify(template_id: &mut [u8]) -> Result<(), Error>;

    /// Aborts a finger identification process.
    fn abort_identify() -> Result<(), Error>;

    /// Deletes a template (or all) from the enrolled fingers.
    ///
    /// If the template ID is not provided, all templates are deleted.
    fn delete_template(template_id: Option<&[u8]>) -> Result<(), Error>;

    /// Returns a concatenation of the template IDs.
    fn list_templates() -> Result<Vec<u8>, Error>;
}
