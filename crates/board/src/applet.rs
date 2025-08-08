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
    /// Installs or uninstalls an applet.
    ///
    /// An empty transfer uninstalls the applet (because valid applets are not empty). A non-empty
    /// transfer installs the transferred applet.
    ///
    /// Calling `start(true)` will uninstall the current applet if any. It is always possible to
    /// call `start()`, in which case a new transfer process is started.
    ///
    /// Until `finish()` is called, there should be no installed applet. In particular, if the
    /// platform reboots and `get()` is called, the returned slice should be empty.
    type Install: crate::transfer::Api;

    /// Returns the persisted applet.
    ///
    /// # Safety
    ///
    /// This function must not be called during a transfer process (unless it's a dry-run). Besides,
    /// whenever `start(true)` is called, it invalidates the result of a previous call to this
    /// function.
    unsafe fn get() -> Result<&'static [u8], Error>;

    /// Notifies an applet start.
    fn notify_start() {}

    /// Notifies an applet exit.
    fn notify_exit(status: ExitStatus) {
        let _ = status;
    }
}

/// Applet install interface.
pub type Install<B> = <super::Applet<B> as Api>::Install;
