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

//! Fingerprint sensor interface.

use crate::{Error, Support};

/// Fingerprint sensor event.
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Debug, PartialEq, Eq)]
pub enum Event {
    /// The image was captured.
    ///
    /// The image can be fetched by calling `read_capture()`. It must be fetched before another
    /// image can be captured.
    CaptureDone,

    /// The capture process encountered an error.
    CaptureError {
        /// The device error.
        error: Error,
    },
}

impl<B: crate::Api> From<Event> for crate::Event<B> {
    fn from(event: Event) -> Self {
        super::Event::Sensor(event).into()
    }
}

/// Fingerprint sensor interface.
pub trait Api: Support<bool> + Send {
    /// Width in bytes of an image.
    const IMAGE_WIDTH: usize;

    /// Height in bytes of an image.
    const IMAGE_HEIGHT: usize;

    /// Starts an image capture process.
    ///
    /// The process sends `CaptureDone` for success and `CaptureError` for error.
    fn start_capture() -> Result<(), Error>;

    /// Reads a captured image.
    fn read_capture(image: &mut [u8]) -> Result<(), Error>;

    /// Aborts an image capture process.
    fn abort_capture() -> Result<(), Error>;
}
