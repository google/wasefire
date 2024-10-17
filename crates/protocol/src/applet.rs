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

use wasefire_wire::Wire;

#[derive(Debug, Wire)]
pub struct Request<'a> {
    pub applet_id: AppletId,
    pub request: &'a [u8],
}

#[derive(Debug, Copy, Clone, Wire)]
pub struct AppletId;

#[derive(Debug, Wire)]
pub struct Tunnel<'a> {
    pub applet_id: AppletId,
    pub delimiter: &'a [u8],
}

#[derive(Debug, Copy, Clone, Wire)]
pub enum ExitStatus {
    /// The applet exited successfully.
    Exit,

    /// The applet aborted (e.g. it panicked).
    Abort,

    /// The applet trapped (e.g. bad memory access).
    Trap,

    /// The applet was killed (e.g. it was uninstalled).
    Kill,
}
