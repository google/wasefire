// Copyright 2024 Google LLC
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

//! Applet interface.

use wasefire_protocol::applet::ExitStatus;

use crate::Error;

/// Applet interface.
pub trait Api: Send {
    /// Returns the persisted applet.
    ///
    /// # Safety
    ///
    /// This function must not be called after `start(true)` until `finish()` is called. Besides,
    /// whenever `start(true)` is called, it invalidates the result of a previous call to this
    /// function.
    unsafe fn get() -> Result<&'static [u8], Error>;

    /// Starts the process of writing an applet to flash.
    ///
    /// Also erases the previous applet (if any). In particular, writing an empty applet can be used
    /// to erase an existing applet (because valid applets are not empty).
    ///
    /// In case an applet is already being written, this function will start a new process.
    ///
    /// During a dry-run, any mutable operation is skipped and only checks are performed.
    fn start(dry_run: bool) -> Result<(), Error>;

    /// Writes the next chunk of an applet being written to flash.
    fn write(chunk: &[u8]) -> Result<(), Error>;

    /// Finishes the process of writing an applet to flash.
    ///
    /// Implementations should make sure that if the platform reboots before this call, the
    /// persisted applet is empty.
    fn finish() -> Result<(), Error>;

    /// Notifies an applet start.
    fn notify_start() {}

    /// Notifies an applet exit.
    fn notify_exit(status: ExitStatus) {
        let _ = status;
    }
}
